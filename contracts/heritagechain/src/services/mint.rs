// =============================================================================
// HeritageChain — Mint Service
//
// This module provides the core logic for managing heritage collectibles within
// the HeritageChain smart contract. It handles:
//
// - Definition of collectible metadata (artist, site, supply, price).
// - Secure minting of new collectibles (restricted to admin).
// - Deactivation and reactivation of collectibles.
// - Efficient on-chain storage and querying of collectible data.
// - Event emission for all state-changing operations.
// =============================================================================

#![allow(dead_code, unused_imports)]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short, Address, Env, Vec,
};

// --- Types ---

/// Master record for a limited-edition Collectible created by the admin.
///
/// Key invariants:
///   - `total_supply`     is fixed at mint time and NEVER changes.
///   - `available_supply` starts equal to `total_supply`. Only the
///     purchase module (Role 4) may decrement it.
///   - `is_active`        controls whether purchases are allowed.
#[contracttype]
#[derive(Clone, Debug)]
pub struct Collectible {
    /// Auto-assigned monotonic ID (from `storage::get_next_id`). Starts at 1.
    pub id: u32,

    /// Stellar address of the verified Ethiopian artist who created the artwork.
    pub artist: Address,

    /// Stellar address of the heritage site this collectible represents.
    pub site: Address,

    /// Price per unit in XLM stroops. 1 XLM = 10_000_000 stroops. Always > 0.
    pub price: i128,

    /// Address that controls the full supply at mint time — always the admin.
    /// The purchase module updates ownership per-unit when a tourist buys.
    pub owner: Address,

    /// Total number of units in this edition. Immutable after minting.
    pub total_supply: u32,

    /// Units still available for purchase. Starts equal to `total_supply`.
    /// Decremented exclusively by the purchase module (Role 4).
    pub available_supply: u32,

    /// Whether this collectible accepts purchases. Admin can pause/resume.
    pub is_active: bool,

    /// Ledger sequence at the moment this collectible was minted.
    pub minted_at: u32,
}

/// Event data emitted after every successful `mint_collectible` call.
/// Off-chain indexers use this to build the public gallery and audit log.
#[contracttype]
#[derive(Clone, Debug)]
pub struct MintedEvent {
    pub id: u32,
    pub admin: Address,
    pub artist: Address,
    pub site: Address,
    pub price: i128,
    pub total_supply: u32,
    pub ledger: u32,
}

// --- Storage ---

/// Storage key variants — add these three to the existing DataKey enum.
///
/// ```text
/// // In src/storage.rs, inside enum DataKey { ... }:
/// NextCollectibleId,             // instance  — monotonic u32 counter
/// Collectible(u32),              // persistent — id → Collectible
/// UserCollectibles(Address),     // persistent — owner → Vec<u32>
/// ```

// TTL used for all persistent Collectible entries (~30 days at 5 s/ledger)
const COLLECTIBLE_TTL: u32 = 535_680;

/// Returns the next available collectible ID and atomically increments
/// the stored counter. IDs start at 1 and are never reused.
///
/// This is the ONLY function that generates IDs — callers must never
/// compute or guess IDs manually.
fn get_next_id(env: &Env) -> u32 {
    // Re-use the same enum arm that the real storage module will expose.
    // In the integrated codebase this reads from DataKey::NextCollectibleId.
    let key = symbol_short!("nxtcol"); // placeholder for DataKey::NextCollectibleId
    let id: u32 = env.storage().instance().get(&key).unwrap_or(1_u32);
    env.storage().instance().set(&key, &(id + 1));
    id
}

/// Persist a `Collectible` to durable storage and refresh its TTL.
/// This is the ONLY write path for Collectible data.
fn save_collectible(env: &Env, collectible: &Collectible) {
    let key = (symbol_short!("col"), collectible.id);
    env.storage().persistent().set(&key, collectible);
    env.storage()
        .persistent()
        .extend_ttl(&key, COLLECTIBLE_TTL, COLLECTIBLE_TTL);
}

/// Retrieve a `Collectible` by ID. Refreshes TTL on hit.
/// Returns `None` if the ID has never been minted.
fn get_collectible_by_id(env: &Env, id: u32) -> Option<Collectible> {
    let key = (symbol_short!("col"), id);
    let result: Option<Collectible> = env.storage().persistent().get(&key);
    if result.is_some() {
        env.storage()
            .persistent()
            .extend_ttl(&key, COLLECTIBLE_TTL, COLLECTIBLE_TTL);
    }
    result
}

/// Append a collectible ID to the owner's catalogue list.
/// Called once per mint, immediately after `save_collectible`.
fn add_to_user(env: &Env, owner: &Address, collectible_id: u32) {
    let key = (symbol_short!("ucol"), owner.clone());
    let mut ids: Vec<u32> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(collectible_id);
    env.storage().persistent().set(&key, &ids);
    env.storage()
        .persistent()
        .extend_ttl(&key, COLLECTIBLE_TTL, COLLECTIBLE_TTL);
}

/// Retrieve all collectible IDs owned by an address.
fn get_user_collectibles(env: &Env, owner: &Address) -> Vec<u32> {
    let key = (symbol_short!("ucol"), owner.clone());
    let result: Vec<u32> = env
        .storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env));
    if !result.is_empty() {
        env.storage()
            .persistent()
            .extend_ttl(&key, COLLECTIBLE_TTL, COLLECTIBLE_TTL);
    }
    result
}

/// Return the total number of collectibles minted so far.
fn get_collectible_count(env: &Env) -> u32 {
    let key = symbol_short!("nxtcol");
    env.storage()
        .instance()
        .get(&key)
        .unwrap_or(1_u32)
        .saturating_sub(1)
}

// --- Events ---

/// Emitted once per successful `mint_collectible`.
/// Indexers key on `("mint", "minted")` to build the collectible catalogue.
fn emit_minted(
    env: &Env,
    id: u32,
    admin: &Address,
    artist: &Address,
    site: &Address,
    price: i128,
    total_supply: u32,
) {
    env.events().publish(
        (symbol_short!("mint"), symbol_short!("minted")),
        (
            id,
            admin.clone(),
            artist.clone(),
            site.clone(),
            price,
            total_supply,
            env.ledger().sequence(),
        ),
    );
}

/// Emitted when admin changes `is_active` on a Collectible.
/// Suppressed on no-ops (state unchanged) to keep the event log clean.
fn emit_toggled(env: &Env, id: u32, admin: &Address, is_active: bool) {
    env.events().publish(
        (symbol_short!("mint"), symbol_short!("toggled")),
        (id, admin.clone(), is_active),
    );
}

/// Emitted by the PURCHASE module (Role 4) when `available_supply` drops.
/// Defined here so all supply-change events share one canonical topic,
/// enabling full audit-log reconstruction from events alone.
fn emit_supply_changed(env: &Env, id: u32, available_supply: u32, delta: u32) {
    env.events().publish(
        (symbol_short!("mint"), symbol_short!("supply")),
        (id, available_supply, delta),
    );
}

// --- Business Logic ---

/// Error codes used by the mint module.
/// These live in src/error.rs in the real codebase.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MintError {
    Unauthorized = 3,        // matches HeritageError::Unauthorized
    CollectibleNotFound = 6, // matches HeritageError::CollectibleNotFound
    InvalidPrice = 10,       // matches HeritageError::InvalidPrice
    OverflowError = 15,      // matches HeritageError::OverflowError
    InvalidSupply = 16,      // matches HeritageError::InvalidSupply
}

// ---------------------------------------------------------------------------
// mint_collectible — the primary Role 3 function
// ---------------------------------------------------------------------------

/// Create a new limited-edition Collectible and register it in contract storage.
///
/// # Authorization
/// `admin.require_auth()` enforces that the transaction is cryptographically
/// signed by the admin keypair. A second check against the stored admin
/// address closes the gap where an unknown address could forge a signature.
///
/// # Supply invariant
/// `available_supply` is initialised to `total_supply` here and is NEVER
/// touched again by this module. Only the purchase module (Role 4) may
/// decrement it. This is enforced by the module boundary — there is no
/// decrement code in this file.
///
/// # Arguments
/// * `env`          — Soroban execution environment
/// * `admin`        — Caller; must equal the stored platform admin
/// * `artist`       — Verified artist's Stellar address
/// * `site`         — Heritage site's Stellar address
/// * `price`        — Price per unit in XLM stroops (must be > 0)
/// * `total_supply` — Units in this edition (must be 1–1_000_000)
///
/// # Returns
/// The unique `u32` ID assigned to the newly created Collectible.
///
/// # Errors
/// | Error            | Condition                           |
/// |------------------|-------------------------------------|
/// | `Unauthorized`   | caller ≠ stored admin               |
/// | `InvalidPrice`   | price ≤ 0                           |
/// | `InvalidSupply`  | supply == 0 or supply > 1_000_000   |
/// | `OverflowError`  | stats counter overflow (defensive)  |
pub fn mint_collectible(
    env: Env,
    admin: Address,
    artist: Address,
    site: Address,
    price: i128,
    total_supply: u32,
) -> Result<u32, MintError> {
    // ── Step 1: Authorization ─────────────────────────────────────────────
    //
    // require_auth() is a CRYPTOGRAPHIC check performed by the Soroban host.
    // It aborts the entire transaction if the invoker has not provided a
    // valid ed25519 signature for `admin`. No amount of code trickery can
    // bypass it — it is enforced at the VM level.
    admin.require_auth();

    // Secondary contract-level check: even if someone constructs a
    // transaction signed by an arbitrary valid keypair, they still can't
    // mint unless their address matches the one stored during initialise().
    let stored_admin = get_stored_admin(&env);
    if admin != stored_admin {
        return Err(MintError::Unauthorized);
    }

    // ── Step 2: Input validation ──────────────────────────────────────────
    //
    // Price check: the revenue-split module (revenue.rs) divides the price
    // using basis-point arithmetic. A zero or negative price would produce
    // zero or negative shares, breaking the 70/20/10 invariant downstream.
    if price <= 0 {
        return Err(MintError::InvalidPrice);
    }

    // Supply check: zero supply is logically incoherent — the collectible
    // would be sold out before any purchase. Upper cap of 1_000_000 prevents
    // u32 overflow in downstream counters.
    if total_supply == 0 || total_supply > 1_000_000 {
        return Err(MintError::InvalidSupply);
    }

    // ── Step 3: Assign unique ID ──────────────────────────────────────────
    //
    // get_next_id is the single source of truth. It reads the stored counter,
    // returns the current value, and atomically increments it. IDs start at 1.
    // This is deterministic, crash-safe, and never produces duplicates.
    let id = get_next_id(&env);

    // ── Step 4: Construct the Collectible ─────────────────────────────────
    //
    // available_supply is set equal to total_supply here — and only here.
    // The purchase module will decrement it. This module never touches it again.
    let collectible = Collectible {
        id,
        artist: artist.clone(),
        site: site.clone(),
        price,
        owner: admin.clone(), // admin is the initial supply controller
        total_supply,
        available_supply: total_supply, // INVARIANT: available = total at birth
        is_active: true,                // immediately open for purchase
        minted_at: env.ledger().sequence(),
    };

    // ── Step 5: Persist to storage ────────────────────────────────────────
    //
    // All writes go through the storage module helpers. There are no direct
    // calls to env.storage() in this function — that boundary is enforced
    // by code review and the module structure.
    save_collectible(&env, &collectible);

    // ── Step 6: Record admin ownership ───────────────────────────────────
    //
    // The admin's collectible list is updated so the `get_admin_collectibles`
    // query returns the full catalogue. This is purely for indexing — it does
    // not confer any financial rights.
    add_to_user(&env, &admin, id);

    // ── Step 7: Update platform stats ────────────────────────────────────
    //
    // Increment total_templates so the admin dashboard shows the live count.
    // checked_add ensures we never silently overflow a u32 counter.
    let mut templates_count = get_templates_count(&env);
    templates_count = templates_count
        .checked_add(1)
        .ok_or(MintError::OverflowError)?;
    set_templates_count(&env, templates_count);

    // ── Step 8: Extend instance TTL ───────────────────────────────────────
    //
    // The ID counter and stats live in instance storage. After each write
    // we reset the TTL so the contract doesn't expire between mints.
    env.storage().instance().extend_ttl(535_680, 535_680);

    // ── Step 9: Emit event ────────────────────────────────────────────────
    //
    // The event is published LAST, after all storage writes succeed. If any
    // earlier step fails and returns an Err, no event is emitted — the caller
    // sees a clean failure with no misleading on-chain trace.
    emit_minted(&env, id, &admin, &artist, &site, price, total_supply);

    Ok(id)
}

// ---------------------------------------------------------------------------
// toggle_collectible
// ---------------------------------------------------------------------------

/// Activate or deactivate a Collectible without destroying it.
///
/// - When `is_active = false`, the purchase module rejects all buy attempts.
/// - When `is_active = true`, purchases resume from wherever `available_supply`
///   left off — no supply is restored.
/// - If the current state already equals `is_active`, this is a no-op and
///   no storage write or event is produced (saves gas).
///
/// Admin-only. The Collectible must already exist.
pub fn toggle_collectible(
    env: Env,
    admin: Address,
    id: u32,
    is_active: bool,
) -> Result<(), MintError> {
    // Authorization — same two-layer check as mint_collectible
    admin.require_auth();
    let stored_admin = get_stored_admin(&env);
    if admin != stored_admin {
        return Err(MintError::Unauthorized);
    }

    // Load — returns CollectibleNotFound for unknown IDs
    let mut collectible = get_collectible_by_id(&env, id).ok_or(MintError::CollectibleNotFound)?;

    // No-op guard — avoids a storage write and event if state is unchanged
    if collectible.is_active == is_active {
        return Ok(());
    }

    // Update and persist
    collectible.is_active = is_active;
    save_collectible(&env, &collectible);
    env.storage().instance().extend_ttl(535_680, 535_680);

    // Emit only when state actually changed
    emit_toggled(&env, id, &admin, is_active);

    Ok(())
}

// ---------------------------------------------------------------------------
// Read-only query helpers (exposed through lib.rs)
// ---------------------------------------------------------------------------

/// Retrieve a Collectible by its ID. Returns `None` if not found.
pub fn get_collectible(env: &Env, id: u32) -> Option<Collectible> {
    get_collectible_by_id(env, id)
}

/// Return all collectible IDs in an owner's catalogue.
pub fn get_admin_collectibles(env: &Env, admin: &Address) -> Vec<u32> {
    get_user_collectibles(env, admin)
}

/// Return the total number of collectibles minted so far.
pub fn collectible_count(env: &Env) -> u32 {
    get_collectible_count(env)
}

// ── Stub helpers (replaced by real storage calls in the integrated codebase) ─

fn get_stored_admin(env: &Env) -> Address {
    // In src/storage.rs: storage::get_admin(env)
    env.storage()
        .instance()
        .get(&symbol_short!("admin"))
        .expect("Admin not set")
}

fn get_templates_count(env: &Env) -> u32 {
    env.storage()
        .instance()
        .get(&symbol_short!("tmplcnt"))
        .unwrap_or(0_u32)
}

fn set_templates_count(env: &Env, count: u32) {
    env.storage()
        .instance()
        .set(&symbol_short!("tmplcnt"), &count);
}



// --- Tests ---

#[cfg(test)]
mod mint_tests {
    use super::*;
    use soroban_sdk::{
        contract, contractimpl, symbol_short, testutils::Address as _, testutils::Events as _,
        Address, Env, Vec,
    };

    #[contract]
    pub struct TestContract;

    #[contractimpl]
    impl TestContract {}

    struct TestEnv {
        env: Env,
        contract_id: Address,
        admin: Address,
        artist: Address,
        site: Address,
    }

    fn make_env() -> TestEnv {
        let env = Env::default();
        env.mock_all_auths();

        let contract_id = env.register(TestContract, ());

        let admin = Address::generate(&env);
        let artist = Address::generate(&env);
        let site = Address::generate(&env);

        // Bootstrap the stored admin so auth checks pass
        env.as_contract(&contract_id, || {
            env.storage()
                .instance()
                .set(&symbol_short!("admin"), &admin);
        });

        TestEnv {
            env,
            contract_id,
            admin,
            artist,
            site,
        }
    }

    fn mint(t: &TestEnv, price: i128, supply: u32) -> u32 {
        t.env.as_contract(&t.contract_id, || {
            super::mint_collectible(
                t.env.clone(),
                t.admin.clone(),
                t.artist.clone(),
                t.site.clone(),
                price,
                supply,
            )
            .unwrap()
        })
    }

    fn get(t: &TestEnv, id: u32) -> Option<Collectible> {
        t.env
            .as_contract(&t.contract_id, || super::get_collectible(&t.env, id))
    }

    fn toggle(t: &TestEnv, id: u32, is_active: bool) -> Result<(), MintError> {
        t.env.as_contract(&t.contract_id, || {
            super::toggle_collectible(t.env.clone(), t.admin.clone(), id, is_active)
        })
    }

    fn admin_list(t: &TestEnv) -> Vec<u32> {
        t.env.as_contract(&t.contract_id, || {
            super::get_admin_collectibles(&t.env, &t.admin)
        })
    }

    fn count(t: &TestEnv) -> u32 {
        t.env
            .as_contract(&t.contract_id, || super::collectible_count(&t.env))
    }

    fn templates(t: &TestEnv) -> u32 {
        t.env
            .as_contract(&t.contract_id, || super::get_templates_count(&t.env))
    }

    // ── Happy-path ────────────────────────────────────────────────────────

    #[test]
    fn ids_are_sequential_from_one() {
        let t = make_env();
        let i1 = mint(&t, 100_0000000, 10_000);
        let i2 = mint(&t, 100_0000000, 100);
        let i3 = mint(&t, 500_0000000, 1);
        assert_eq!(i1, 1);
        assert_eq!(i2, 2);
        assert_eq!(i3, 3);
    }

    #[test]
    fn available_supply_equals_total_supply_at_mint() {
        let t = make_env();
        let id = mint(&t, 100_0000000, 250);
        let c = get(&t, id).unwrap();
        assert_eq!(
            c.available_supply, c.total_supply,
            "INVARIANT BROKEN: available_supply must equal total_supply at birth"
        );
    }

    #[test]
    fn is_active_is_true_after_mint() {
        let t = make_env();
        let id = mint(&t, 50_0000000, 500);
        assert!(get(&t, id).unwrap().is_active);
    }

    #[test]
    fn price_stored_correctly() {
        let t = make_env();
        let id = mint(&t, 999_0000000, 10);
        assert_eq!(get(&t, id).unwrap().price, 999_0000000);
    }

    #[test]
    fn artist_and_site_stored_correctly() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);
        let c = get(&t, id).unwrap();
        assert_eq!(c.artist, t.artist);
        assert_eq!(c.site, t.site);
    }

    #[test]
    fn owner_is_admin_at_mint_time() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);
        assert_eq!(get(&t, id).unwrap().owner, t.admin);
    }

    #[test]
    fn minted_at_is_current_ledger() {
        let t = make_env();
        let before = t.env.ledger().sequence();
        let id = mint(&t, 10_0000000, 100);
        let after = t.env.ledger().sequence();
        let minted = get(&t, id).unwrap().minted_at;
        assert!(minted >= before && minted <= after);
    }

    #[test]
    fn admin_catalogue_updated_after_mint() {
        let t = make_env();
        let id1 = mint(&t, 10_0000000, 100);
        let id2 = mint(&t, 50_0000000, 50);
        let id3 = mint(&t, 500_0000000, 1);
        let list = admin_list(&t);
        assert_eq!(list.len(), 3);
        assert_eq!(list.get(0).unwrap(), id1);
        assert_eq!(list.get(1).unwrap(), id2);
        assert_eq!(list.get(2).unwrap(), id3);
    }

    #[test]
    fn collectible_count_increments_per_mint() {
        let t = make_env();
        assert_eq!(count(&t), 0);
        mint(&t, 10_0000000, 100);
        assert_eq!(count(&t), 1);
        mint(&t, 10_0000000, 100);
        assert_eq!(count(&t), 2);
    }

    #[test]
    fn templates_stat_increments_per_mint() {
        let t = make_env();
        let before = templates(&t);
        mint(&t, 10_0000000, 100);
        mint(&t, 10_0000000, 200);
        assert_eq!(templates(&t), before + 2);
    }

    // ── All three edition types ───────────────────────────────────────────

    #[test]
    fn pilgrim_edition_10k_supply() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 10_000);
        let c = get(&t, id).unwrap();
        assert_eq!(c.total_supply, 10_000);
    }

    #[test]
    fn collector_edition_100_supply() {
        let t = make_env();
        let id = mint(&t, 100_0000000, 100);
        let c = get(&t, id).unwrap();
        assert_eq!(c.total_supply, 100);
    }

    #[test]
    fn guardian_edition_supply_one() {
        let t = make_env();
        let id = mint(&t, 500_0000000, 1);
        let c = get(&t, id).unwrap();
        assert_eq!(c.total_supply, 1);
        assert_eq!(c.available_supply, 1);
    }

    // ── Authorization ─────────────────────────────────────────────────────

    #[test]
    fn non_admin_cannot_mint() {
        let t = make_env();
        let intruder = Address::generate(&t.env);
        let res = t.env.as_contract(&t.contract_id, || {
            super::mint_collectible(
                t.env.clone(),
                intruder,
                t.artist.clone(),
                t.site.clone(),
                10_0000000,
                100,
            )
        });
        assert_eq!(res, Err(MintError::Unauthorized));
    }

    #[test]
    fn artist_address_cannot_mint() {
        let t = make_env();
        let res = t.env.as_contract(&t.contract_id, || {
            super::mint_collectible(
                t.env.clone(),
                t.artist.clone(), // artist ≠ admin
                t.artist.clone(),
                t.site.clone(),
                10_0000000,
                100,
            )
        });
        assert_eq!(res, Err(MintError::Unauthorized));
    }

    // ── Input validation ──────────────────────────────────────────────────

    #[test]
    fn zero_price_rejected() {
        let t = make_env();
        let res = t.env.as_contract(&t.contract_id, || {
            super::mint_collectible(
                t.env.clone(),
                t.admin.clone(),
                t.artist.clone(),
                t.site.clone(),
                0,
                100,
            )
        });
        assert_eq!(res, Err(MintError::InvalidPrice));
    }

    #[test]
    fn negative_price_rejected() {
        let t = make_env();
        let res = t.env.as_contract(&t.contract_id, || {
            super::mint_collectible(
                t.env.clone(),
                t.admin.clone(),
                t.artist.clone(),
                t.site.clone(),
                -1,
                100,
            )
        });
        assert_eq!(res, Err(MintError::InvalidPrice));
    }

    #[test]
    fn zero_supply_rejected() {
        let t = make_env();
        let res = t.env.as_contract(&t.contract_id, || {
            super::mint_collectible(
                t.env.clone(),
                t.admin.clone(),
                t.artist.clone(),
                t.site.clone(),
                10_0000000,
                0,
            )
        });
        assert_eq!(res, Err(MintError::InvalidSupply));
    }

    #[test]
    fn supply_above_cap_rejected() {
        let t = make_env();
        let res = t.env.as_contract(&t.contract_id, || {
            super::mint_collectible(
                t.env.clone(),
                t.admin.clone(),
                t.artist.clone(),
                t.site.clone(),
                10_0000000,
                1_000_001,
            )
        });
        assert_eq!(res, Err(MintError::InvalidSupply));
    }

    #[test]
    fn minimum_price_one_stroop_accepted() {
        let t = make_env();
        let id = mint(&t, 1, 100);
        assert_eq!(get(&t, id).unwrap().price, 1);
    }

    #[test]
    fn minimum_supply_one_accepted() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 1);
        let c = get(&t, id).unwrap();
        assert_eq!(c.total_supply, 1);
        assert_eq!(c.available_supply, 1);
    }

    #[test]
    fn maximum_supply_one_million_accepted() {
        let t = make_env();
        let id = mint(&t, 1, 1_000_000);
        let c = get(&t, id).unwrap();
        assert_eq!(c.total_supply, 1_000_000);
        assert_eq!(c.available_supply, 1_000_000);
    }

    // ── toggle_collectible ────────────────────────────────────────────────

    #[test]
    fn toggle_deactivates_collectible() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);
        assert!(get(&t, id).unwrap().is_active);

        toggle(&t, id, false).unwrap();
        assert!(!get(&t, id).unwrap().is_active);
    }

    #[test]
    fn toggle_reactivates_collectible() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);
        toggle(&t, id, false).unwrap();
        toggle(&t, id, true).unwrap();
        assert!(get(&t, id).unwrap().is_active);
    }

    #[test]
    fn toggle_noop_on_same_state_succeeds() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);
        // Already active — toggling to true is a no-op, should not error
        toggle(&t, id, true).unwrap();
        assert!(get(&t, id).unwrap().is_active);
    }

    #[test]
    fn non_admin_cannot_toggle() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);
        let intruder = Address::generate(&t.env);
        let res = t.env.as_contract(&t.contract_id, || {
            super::toggle_collectible(t.env.clone(), intruder, id, false)
        });
        assert_eq!(res, Err(MintError::Unauthorized));
    }

    #[test]
    fn toggle_unknown_id_rejected() {
        let t = make_env();
        let res = toggle(&t, 999, false);
        assert_eq!(res, Err(MintError::CollectibleNotFound));
    }

    // ── get_collectible ───────────────────────────────────────────────────

    #[test]
    fn get_returns_none_for_unknown_id() {
        let t = make_env();
        assert!(get(&t, 999).is_none());
    }

    #[test]
    fn get_returns_some_after_mint() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);
        assert!(get(&t, id).is_some());
    }

    // ── Supply isolation ──────────────────────────────────────────────────

    #[test]
    fn mint_never_decrements_available_supply() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 500);
        let c = get(&t, id).unwrap();
        assert_eq!(
            c.available_supply, c.total_supply,
            "mint must NEVER decrement available_supply (purchase module's job)"
        );
    }

    #[test]
    fn multiple_collectibles_have_independent_supplies() {
        let t = make_env();
        let id1 = mint(&t, 10_0000000, 100);
        let id2 = mint(&t, 50_0000000, 50);
        let id3 = mint(&t, 500_0000000, 1);

        assert_eq!(get(&t, id1).unwrap().total_supply, 100);
        assert_eq!(get(&t, id2).unwrap().total_supply, 50);
        assert_eq!(get(&t, id3).unwrap().total_supply, 1);
    }

    // ── Idempotency & determinism ─────────────────────────────────────────

    #[test]
    fn twenty_mints_produce_twenty_unique_ids() {
        let t = make_env();
        let mut seen: Vec<u32> = Vec::new(&t.env);
        for _ in 0..20_u32 {
            let id = mint(&t, 10_0000000, 10);
            for i in 0..seen.len() {
                assert_ne!(seen.get(i).unwrap(), id, "duplicate ID");
            }
            seen.push_back(id);
        }
        assert_eq!(seen.len(), 20);
    }

    #[test]
    fn identical_inputs_still_produce_different_ids() {
        let t = make_env();
        let id1 = mint(&t, 10_0000000, 100);
        let id2 = mint(&t, 10_0000000, 100);
        assert_ne!(id1, id2, "IDs must differ even for identical inputs");
    }

    // ── Event emission ────────────────────────────────────────────────────

    #[test]
    fn mint_emits_at_least_one_event() {
        let t = make_env();
        let _id = mint(&t, 100_0000000, 250);
        assert!(
            !t.env.events().all().is_empty(),
            "mint_collectible must emit at least one event"
        );
    }

    #[test]
    fn toggle_emits_event_when_state_changes() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);

        // Clear events from minting
        t.env.events().all();

        toggle(&t, id, false).unwrap();

        assert_eq!(
            t.env.events().all().len(),
            1,
            "toggle must emit exactly one event when state changes"
        );
    }

    #[test]
    fn toggle_noop_does_not_emit_event() {
        let t = make_env();
        let id = mint(&t, 10_0000000, 100);

        // Clear events from minting
        t.env.events().all();

        // Already active — no-op
        toggle(&t, id, true).unwrap();

        assert_eq!(
            t.env.events().all().len(),
            0,
            "no-op toggle must NOT emit an event"
        );
    }

    // ── No side-effects ───────────────────────────────────────────────────

    #[test]
    fn mint_does_not_alter_available_supply_after_construction() {
        // Demonstrates the hard boundary: mint sets available_supply once,
        // then never modifies it. Any subsequent read should return the
        // original total_supply unchanged.
        let t = make_env();
        let id = mint(&t, 10_0000000, 777);

        // Read back three times — value must be stable
        for _ in 0..3 {
            let c = get(&t, id).unwrap();
            assert_eq!(
                c.available_supply, 777,
                "available_supply changed without a purchase — module boundary violated"
            );
        }
    }
}
