use crate::schedule::SchedulerFlags;
#[cfg(feature = "stress")]
use crate::stress_nifs::ValgrindTestResource;
use crate::{Env, NifOutcome, Resource, ResourceArc, Term, gleam_nif, init_nifs};
use crate::{NifRecord, NifUnitEnum};
use std::collections::{BTreeMap, BTreeSet, HashSet, LinkedList, VecDeque};
use std::net::{IpAddr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::sync::atomic::{AtomicI64, Ordering};
use std::time::SystemTime;

pub struct Counter {
    current: AtomicI64,
    target: i64,
}

impl Resource for Counter {}

/// A user record synchronized between Rust and Gleam
#[derive(NifRecord, Debug, PartialEq, Clone)]
#[tag = "user"]
pub struct User {
    pub id: i64,
    pub name: String,
    pub is_active: bool,
}

/// Status enum synchronized between Rust and Gleam
#[derive(NifUnitEnum, Debug, PartialEq, Eq, Copy, Clone)]
pub enum UserStatus {
    Pending,
    Active,
    Banned,
}

#[gleam_nif]
pub fn make_user(id: i64, name: String) -> User {
    User {
        id,
        name,
        is_active: true,
    }
}

#[gleam_nif]
pub fn user_get_name(user: User) -> String {
    user.name
}

#[gleam_nif]
pub fn is_user_banned(status: UserStatus) -> bool {
    matches!(status, UserStatus::Banned)
}

#[gleam_nif]
pub fn counter_new(target: i64) -> ResourceArc<Counter> {
    ResourceArc::new(Counter {
        current: AtomicI64::new(0),
        target,
    })
}

#[gleam_nif]
pub fn counter_read(counter: ResourceArc<Counter>) -> i64 {
    counter.current.load(Ordering::SeqCst)
}

#[gleam_nif]
pub fn cooperative_count(counter: ResourceArc<Counter>) -> NifOutcome<i64> {
    loop {
        let val = counter.current.load(Ordering::SeqCst);
        if val >= counter.target {
            return NifOutcome::Done(val);
        }
        let next = counter.current.fetch_add(1, Ordering::SeqCst) + 1;
        if next >= counter.target {
            return NifOutcome::Done(next);
        }
        if next % 500 == 0 {
            return NifOutcome::Yield(SchedulerFlags::Normal);
        }
    }
}

fn on_load(env: Env, _info: Term) -> bool {
    #[allow(unused_mut)]
    let mut ok = env.register::<Counter>().is_ok();
    #[cfg(feature = "stress")]
    {
        ok = ok && env.register::<ValgrindTestResource>().is_ok();
    }
    ok
}

#[gleam_nif]
pub fn add(a: i64, b: i64) -> i64 {
    a + b
}

#[gleam_nif]
pub fn sub(a: i64, b: i64) -> i64 {
    a - b
}

#[gleam_nif]
pub fn greet(name: String) -> String {
    format!("Hello, {}!", name)
}

#[gleam_nif]
pub fn double_list(items: Vec<i64>) -> Vec<i64> {
    items.into_iter().map(|x| x * 2).collect()
}

#[gleam_nif]
pub fn is_positive(n: i64) -> bool {
    n > 0
}

#[gleam_nif]
pub fn divide(a: f64, b: f64) -> f64 {
    a / b
}

#[gleam_nif]
pub fn make_pair(a: i64, b: String) -> (i64, String) {
    (a, b)
}

#[gleam_nif]
pub fn factorial(n: i64) -> i64 {
    if n <= 1 {
        return 1;
    }
    let mut acc = 1i64;
    for i in 2..=n {
        acc = acc.wrapping_mul(i);
    }
    acc
}

#[gleam_nif(dirty_cpu)]
pub fn fib(n: i64) -> i64 {
    if n <= 1 { n } else { fib(n - 1) + fib(n - 2) }
}

#[gleam_nif]
pub fn echo_i128(n: i128) -> i128 {
    n
}

#[gleam_nif]
pub fn echo_u128(n: u128) -> u128 {
    n
}

#[gleam_nif]
pub fn mul(a: i64, b: i64) -> i64 {
    a * b
}

#[doc(hidden)]
pub mod __generated_registry {
    include!(concat!(env!("OUT_DIR"), "/nif_registry.rs"));
}

#[gleam_nif]
pub fn net_ip_roundtrip(ip: IpAddr) -> IpAddr {
    ip
}

#[gleam_nif]
pub fn net_socket_roundtrip(addr: SocketAddr) -> SocketAddr {
    addr
}

#[gleam_nif]
pub fn net_socket_v4_roundtrip(addr: SocketAddrV4) -> SocketAddrV4 {
    addr
}

#[gleam_nif]
pub fn net_socket_v6_roundtrip(addr: SocketAddrV6) -> SocketAddrV6 {
    addr
}

#[gleam_nif]
pub fn collections_hashset_roundtrip(set: HashSet<i64>) -> HashSet<i64> {
    set
}

#[gleam_nif]
pub fn collections_btreeset_roundtrip(set: BTreeSet<String>) -> BTreeSet<String> {
    set
}

#[gleam_nif]
pub fn collections_vecdeque_roundtrip(dq: VecDeque<i64>) -> VecDeque<i64> {
    dq
}

#[gleam_nif]
pub fn collections_linkedlist_roundtrip(ll: LinkedList<bool>) -> LinkedList<bool> {
    ll
}

#[gleam_nif]
pub fn collections_btreemap_roundtrip(map: BTreeMap<String, i64>) -> BTreeMap<String, i64> {
    map
}

#[gleam_nif]
pub fn time_system_time_roundtrip(t: SystemTime) -> SystemTime {
    t
}

init_nifs!(load = on_load);
