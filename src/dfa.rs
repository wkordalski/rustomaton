use crate::automaton::{Automata, Automaton, Buildable};
use crate::nfa::{ToNfa, NFA};
use crate::regex::{Regex, ToRegex};
use std::cmp::{Ordering, Ordering::*, PartialEq, PartialOrd};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::ops::{Add, Mul, Neg, Not, RangeBounds, Sub};
use std::str::FromStr;

/// https://en.wikipedia.org/wiki/Deterministic_finite_automaton
#[derive(Debug, Clone)]
pub struct DFA<V: Eq + Hash + Display + Copy + Clone + Debug> {
    pub(crate) alphabet: HashSet<V>,
    pub(crate) initial: usize,
    pub(crate) finals: HashSet<usize>,
    pub(crate) transitions: Vec<HashMap<V, usize>>,
}

/// An interface for structs that can be converted into a DFA.
pub trait ToDfa<V: Eq + Hash + Display + Copy + Clone + Debug> {
    fn to_dfa(&self) -> DFA<V>;
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> DFA<V> {
    pub fn intersect(self, b: DFA<V>) -> DFA<V> {
        self.negate().unite(b.negate()).negate()
    }

    /// The algorithm used is https://en.wikipedia.org/wiki/DFA_minimization#Brzozowski's_algorithm.
    pub fn minimize(self) -> DFA<V> {
        self.reverse().to_dfa().reverse().to_dfa()
    }

    /// A contains B if and only if for each `word` w, if B `accepts` w then A `accepts` w.
    pub fn contains(&self, b: &DFA<V>) -> bool {
        self.to_nfa().contains(&b.to_nfa())
    }

    /// Export to dotfile in dots/automaton/i.dot
    pub fn write_dot(&self, n: u8) -> Result<(), std::io::Error> {
        self.to_nfa().write_dot(n)
    }

    pub fn new_empty(alphabet: &HashSet<V>) -> DFA<V> {
        DFA {
            alphabet: alphabet.clone(),
            initial: 0,
            finals: HashSet::new(),
            transitions: vec![HashMap::new()],
        }
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> Automata<V> for DFA<V> {
    fn run(&self, v: &Vec<V>) -> bool {
        let mut actual = self.initial;
        for l in v {
            if let Some(t) = self.transitions[actual].get(l) {
                actual = *t;
            } else {
                return false;
            }
        }
        return self.finals.contains(&actual);
    }

    fn is_complete(&self) -> bool {
        for map in &self.transitions {
            for v in &self.alphabet {
                if !map.contains_key(&v) {
                    return false;
                }
            }
        }
        return true;
    }

    fn is_reachable(&self) -> bool {
        let mut stack = vec![self.initial];
        let mut acc = HashSet::new();
        acc.insert(self.initial);
        while let Some(e) = stack.pop() {
            for (_, v) in &self.transitions[e] {
                if !acc.contains(&v) {
                    acc.insert(*v);
                    stack.push(*v);
                }
            }
        }
        return acc.len() == self.transitions.len();
    }

    fn is_coreachable(&self) -> bool {
        self.to_nfa().is_coreachable()
    }

    fn is_trimmed(&self) -> bool {
        self.to_nfa().is_trimmed()
    }

    fn is_empty(&self) -> bool {
        self.to_nfa().is_empty()
    }

    fn is_full(&self) -> bool {
        self.to_nfa().is_full()
    }

    fn negate(mut self) -> DFA<V> {
        self = self.complete();
        self.finals = (0..self.transitions.len())
            .into_iter()
            .filter(|x| !self.finals.contains(&x))
            .collect();
        self
    }

    fn complete(mut self) -> DFA<V> {
        if self.is_complete() {
            return self;
        }

        let l = self.transitions.len();
        self.transitions.push(HashMap::new());
        for map in &mut self.transitions {
            for v in &self.alphabet {
                if !map.contains_key(&v) {
                    map.insert(*v, l);
                }
            }
        }

        self
    }

    fn make_reachable(self) -> DFA<V> {
        self.to_nfa().make_reachable().to_dfa()
    }

    fn make_coreachable(self) -> DFA<V> {
        self.to_nfa().make_coreachable().to_dfa()
    }

    fn trim(self) -> DFA<V> {
        self.to_nfa().trim().to_dfa()
    }

    fn reverse(self) -> DFA<V> {
        self.to_nfa().reverse().to_dfa()
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> Buildable<V> for DFA<V> {
    fn unite(self, b: DFA<V>) -> DFA<V> {
        self.to_nfa().unite(b.to_nfa()).to_dfa()
    }

    fn concatenate(self, b: DFA<V>) -> DFA<V> {
        self.to_nfa().concatenate(b.to_nfa()).to_dfa()
    }

    fn kleene(self) -> DFA<V> {
        self.to_nfa().kleene().to_dfa()
    }

    fn at_most(self, u: usize) -> DFA<V> {
        self.to_nfa().at_most(u).to_dfa()
    }

    fn at_least(self, u: usize) -> DFA<V> {
        self.to_nfa().at_least(u).to_dfa()
    }

    fn repeat<R: RangeBounds<usize>>(self, r: R) -> DFA<V> {
        self.to_nfa().repeat(r).to_dfa()
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> ToDfa<V> for DFA<V> {
    fn to_dfa(&self) -> DFA<V> {
        self.clone()
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> ToRegex<V> for DFA<V> {
    fn to_regex(&self) -> Regex<V> {
        unimplemented!()
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> ToNfa<V> for DFA<V> {
    fn to_nfa(&self) -> NFA<V> {
        let mut initials = HashSet::new();
        initials.insert(self.initial);
        let mut transitions = Vec::new();
        for map in &self.transitions {
            transitions.push(map.iter().map(|(k, v)| (*k, vec![*v])).collect());
        }
        NFA {
            alphabet: self.alphabet.clone(),
            initials,
            finals: self.finals.clone(),
            transitions,
        }
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> PartialEq<DFA<V>> for DFA<V> {
    fn eq(&self, b: &DFA<V>) -> bool {
        self.le(&b) && self.ge(&b)
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> PartialEq<NFA<V>> for DFA<V> {
    fn eq(&self, b: &NFA<V>) -> bool {
        self.to_nfa().eq(b)
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> PartialEq<Regex<V>> for DFA<V> {
    fn eq(&self, b: &Regex<V>) -> bool {
        self.to_nfa().eq(&b.to_nfa())
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> PartialEq<Automaton<V>> for DFA<V> {
    fn eq(&self, b: &Automaton<V>) -> bool {
        match b {
            Automaton::DFA(v) => self.eq(&*v),
            Automaton::NFA(v) => self.eq(&*v),
            Automaton::REG(v) => self.eq(&*v),
        }
    }
}

impl<V: Eq + Hash + Display + Copy + Clone + Debug> PartialOrd for DFA<V> {
    fn partial_cmp(&self, other: &DFA<V>) -> Option<Ordering> {
        match (self.ge(&other), self.le(&other)) {
            (true, true) => Some(Equal),
            (true, false) => Some(Greater),
            (false, true) => Some(Less),
            (false, false) => None,
        }
    }

    fn lt(&self, other: &DFA<V>) -> bool {
        other.contains(&self) && !self.contains(&other)
    }

    fn le(&self, other: &DFA<V>) -> bool {
        other.contains(&self)
    }

    fn gt(&self, other: &DFA<V>) -> bool {
        self.contains(&other) && !other.contains(&self)
    }

    fn ge(&self, other: &DFA<V>) -> bool {
        self.contains(&other)
    }
}

impl FromStr for DFA<char> {
    type Err = String;

    fn from_str(s: &str) -> Result<DFA<char>, Self::Err> {
        NFA::from_str(s).map(|x| x.to_dfa())
    }
}

/// The multiplication of A and B is A.concatenate(B)
impl<V: Eq + Hash + Display + Copy + Clone + Debug> Mul for DFA<V> {
    type Output = Self;

    fn mul(self, other: DFA<V>) -> DFA<V> {
        self.concatenate(other)
    }
}

/// The negation of A is A.negate().
impl<V: Eq + Hash + Display + Copy + Clone + Debug> Neg for DFA<V> {
    type Output = Self;

    fn neg(self) -> DFA<V> {
        self.negate()
    }
}

/// The opposite of A is A.reverse().
impl<V: Eq + Hash + Display + Copy + Clone + Debug> Not for DFA<V> {
    type Output = Self;

    fn not(self) -> DFA<V> {
        self.reverse()
    }
}

/// The substraction of A and B is an automaton that accepts a word if and only if A accepts it and B doesn't.
impl<V: Eq + Hash + Display + Copy + Clone + Debug> Sub for DFA<V> {
    type Output = Self;

    fn sub(self, other: DFA<V>) -> DFA<V> {
        self.intersect(other.negate())
    }
}

/// The addition fo A and B is an automaton that accepts a word if and only if A or B accept it.
impl<V: Eq + Hash + Display + Copy + Clone + Debug> Add for DFA<V> {
    type Output = Self;

    fn add(self, other: DFA<V>) -> DFA<V> {
        self.unite(other)
    }
}
