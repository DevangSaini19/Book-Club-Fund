#![no_std]
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Map, String, Symbol, Vec,
};
 
// ── Storage keys ──────────────────────────────────────────────────────────────
const BOOKS: Symbol = symbol_short!("BOOKS");
const MEMBERS: Symbol = symbol_short!("MEMBERS");
const TREASURY: Symbol = symbol_short!("TREASURY");
const ADMIN: Symbol = symbol_short!("ADMIN");
const BOOK_CTR: Symbol = symbol_short!("BOOK_CTR");
 
// ── Data types ────────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum BookStatus {
    Available,
    CheckedOut,
    Lost,
}
 
#[contracttype]
#[derive(Clone, Debug)]
pub struct Book {
    pub id: u32,
    pub title: String,
    pub author: String,
    pub funded_by: Address,       // member who proposed / paid for the book
    pub cost: i128,               // cost in stroops (1 XLM = 10_000_000 stroops)
    pub status: BookStatus,
    pub checked_out_by: Option<Address>,
    pub times_read: u32,
}
 
#[contracttype]
#[derive(Clone, Debug)]
pub struct Member {
    pub address: Address,
    pub total_contributed: i128,  // cumulative contributions in stroops
    pub books_read: u32,
    pub active: bool,
}
 
// ── Contract ──────────────────────────────────────────────────────────────────
#[contract]
pub struct BookClubContract;
 
#[contractimpl]
impl BookClubContract {
    // ── Init ──────────────────────────────────────────────────────────────────
 
    /// Deploy the contract. Caller becomes admin.
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&ADMIN) {
            panic!("already initialized");
        }
        admin.require_auth();
        env.storage().instance().set(&ADMIN, &admin);
        env.storage().instance().set(&TREASURY, &0_i128);
        env.storage().instance().set(&BOOK_CTR, &0_u32);
        env.storage()
            .instance()
            .set(&MEMBERS, &Map::<Address, Member>::new(&env));
        env.storage()
            .instance()
            .set(&BOOKS, &Map::<u32, Book>::new(&env));
    }
 
    // ── Membership ────────────────────────────────────────────────────────────
 
    /// Join the club (admin must approve; any member can call this for themselves).
    pub fn join(env: Env, member: Address) {
        member.require_auth();
        let mut members: Map<Address, Member> =
            env.storage().instance().get(&MEMBERS).unwrap();
 
        if members.contains_key(member.clone()) {
            panic!("already a member");
        }
 
        members.set(
            member.clone(),
            Member {
                address: member,
                total_contributed: 0,
                books_read: 0,
                active: true,
            },
        );
        env.storage().instance().set(&MEMBERS, &members);
    }
 
    /// Contribute XLM to the treasury (amount in stroops).
    pub fn contribute(env: Env, member: Address, amount: i128) {
        member.require_auth();
        assert!(amount > 0, "amount must be positive");
 
        let mut members: Map<Address, Member> =
            env.storage().instance().get(&MEMBERS).unwrap();
        let mut m = members.get(member.clone()).expect("not a member");
        assert!(m.active, "member is inactive");
 
        m.total_contributed += amount;
        members.set(member, m);
        env.storage().instance().set(&MEMBERS, &members);
 
        let treasury: i128 = env.storage().instance().get(&TREASURY).unwrap();
        env.storage()
            .instance()
            .set(&TREASURY, &(treasury + amount));
    }
 
    // ── Book management ───────────────────────────────────────────────────────
 
    /// Propose and add a book; deducts cost from treasury.
    pub fn add_book(
        env: Env,
        proposer: Address,
        title: String,
        author: String,
        cost: i128,
    ) -> u32 {
        proposer.require_auth();
        let members: Map<Address, Member> =
            env.storage().instance().get(&MEMBERS).unwrap();
        assert!(members.contains_key(proposer.clone()), "not a member");
 
        let treasury: i128 = env.storage().instance().get(&TREASURY).unwrap();
        assert!(treasury >= cost, "insufficient treasury funds");
 
        let mut counter: u32 = env.storage().instance().get(&BOOK_CTR).unwrap();
        counter += 1;
 
        let book = Book {
            id: counter,
            title,
            author,
            funded_by: proposer,
            cost,
            status: BookStatus::Available,
            checked_out_by: None,
            times_read: 0,
        };
 
        let mut books: Map<u32, Book> =
            env.storage().instance().get(&BOOKS).unwrap();
        books.set(counter, book);
 
        env.storage().instance().set(&BOOKS, &books);
        env.storage()
            .instance()
            .set(&TREASURY, &(treasury - cost));
        env.storage().instance().set(&BOOK_CTR, &counter);
 
        counter // return the new book id
    }
 
    /// Check out a book.
    pub fn checkout(env: Env, member: Address, book_id: u32) {
        member.require_auth();
        let members: Map<Address, Member> =
            env.storage().instance().get(&MEMBERS).unwrap();
        assert!(members.contains_key(member.clone()), "not a member");
 
        let mut books: Map<u32, Book> =
            env.storage().instance().get(&BOOKS).unwrap();
        let mut book = books.get(book_id).expect("book not found");
        assert!(
            book.status == BookStatus::Available,
            "book is not available"
        );
 
        book.status = BookStatus::CheckedOut;
        book.checked_out_by = Some(member);
        books.set(book_id, book);
        env.storage().instance().set(&BOOKS, &books);
    }
 
    /// Return a book and increment read counter.
    pub fn return_book(env: Env, member: Address, book_id: u32) {
        member.require_auth();
        let mut members: Map<Address, Member> =
            env.storage().instance().get(&MEMBERS).unwrap();
 
        let mut books: Map<u32, Book> =
            env.storage().instance().get(&BOOKS).unwrap();
        let mut book = books.get(book_id).expect("book not found");
 
        assert!(
            book.checked_out_by == Some(member.clone()),
            "book not checked out by you"
        );
 
        book.status = BookStatus::Available;
        book.checked_out_by = None;
        book.times_read += 1;
        books.set(book_id, book);
        env.storage().instance().set(&BOOKS, &books);
 
        // credit the reader
        let mut m = members.get(member.clone()).expect("not a member");
        m.books_read += 1;
        members.set(member, m);
        env.storage().instance().set(&MEMBERS, &members);
    }
 
    /// Mark a book as lost (admin only).
    pub fn mark_lost(env: Env, admin: Address, book_id: u32) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&ADMIN).unwrap();
        assert!(admin == stored_admin, "not admin");
 
        let mut books: Map<u32, Book> =
            env.storage().instance().get(&BOOKS).unwrap();
        let mut book = books.get(book_id).expect("book not found");
        book.status = BookStatus::Lost;
        books.set(book_id, book);
        env.storage().instance().set(&BOOKS, &books);
    }
 
    // ── Queries ───────────────────────────────────────────────────────────────
 
    pub fn get_book(env: Env, book_id: u32) -> Book {
        let books: Map<u32, Book> = env.storage().instance().get(&BOOKS).unwrap();
        books.get(book_id).expect("book not found")
    }
 
    pub fn get_member(env: Env, member: Address) -> Member {
        let members: Map<Address, Member> =
            env.storage().instance().get(&MEMBERS).unwrap();
        members.get(member).expect("member not found")
    }
 
    pub fn get_treasury(env: Env) -> i128 {
        env.storage().instance().get(&TREASURY).unwrap()
    }
 
    pub fn get_book_count(env: Env) -> u32 {
        env.storage().instance().get(&BOOK_CTR).unwrap()
    }
 
    /// List all book IDs (caller iterates 1..=get_book_count()).
    pub fn list_available_books(env: Env) -> Vec<u32> {
        let books: Map<u32, Book> = env.storage().instance().get(&BOOKS).unwrap();
        let count: u32 = env.storage().instance().get(&BOOK_CTR).unwrap();
        let mut result = Vec::new(&env);
        for i in 1..=count {
            if let Some(b) = books.get(i) {
                if b.status == BookStatus::Available {
                    result.push_back(i);
                }
            }
        }
        result
    }
}
 
// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;
 
    fn setup() -> (Env, BookClubContractClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(BookClubContract, ());
        let client = BookClubContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin);
        (env, client, admin)
    }
 
    #[test]
    fn test_join_and_contribute() {
        let (env, client, _admin) = setup();
        let alice = Address::generate(&env);
        client.join(&alice);
        client.contribute(&alice, &5_000_000); // 0.5 XLM
        let m = client.get_member(&alice);
        assert_eq!(m.total_contributed, 5_000_000);
        assert_eq!(client.get_treasury(), 5_000_000);
    }
 
    #[test]
    fn test_add_and_checkout_book() {
        let (env, client, _admin) = setup();
        let alice = Address::generate(&env);
        client.join(&alice);
        client.contribute(&alice, &20_000_000); // 2 XLM
 
        let title = soroban_sdk::String::from_str(&env, "Dune");
        let author = soroban_sdk::String::from_str(&env, "Frank Herbert");
        let book_id = client.add_book(&alice, &title, &author, &10_000_000);
 
        assert_eq!(client.get_treasury(), 10_000_000);
 
        client.checkout(&alice, &book_id);
        let book = client.get_book(&book_id);
        assert_eq!(book.status, BookStatus::CheckedOut);
 
        client.return_book(&alice, &book_id);
        let book = client.get_book(&book_id);
        assert_eq!(book.status, BookStatus::Available);
        assert_eq!(book.times_read, 1);
 
        let m = client.get_member(&alice);
        assert_eq!(m.books_read, 1);
    }
 
    #[test]
    fn test_list_available_books() {
        let (env, client, _admin) = setup();
        let alice = Address::generate(&env);
        client.join(&alice);
        client.contribute(&alice, &50_000_000);
 
        let t1 = soroban_sdk::String::from_str(&env, "Book One");
        let a1 = soroban_sdk::String::from_str(&env, "Author A");
        let t2 = soroban_sdk::String::from_str(&env, "Book Two");
        let a2 = soroban_sdk::String::from_str(&env, "Author B");
 
        let id1 = client.add_book(&alice, &t1, &a1, &5_000_000);
        let id2 = client.add_book(&alice, &t2, &a2, &5_000_000);
 
        client.checkout(&alice, &id1);
 
        let available = client.list_available_books();
        assert!(!available.contains(&id1));
        assert!(available.contains(&id2));
    }
}