---
layout:   post
title:    "Implementing Finite Automata (Part 1)"
date:     2016-10-03
category: CompSci
tags:     [automata, computation, finite automata, nfa, dfa, powerset construction, regular language, rust]
---

This is post number three in a [series]({% post_url 2016-03-28-theory-of-computation %}) on Automata (in the formal languages / regex / parsing sense). This is the promised "implementation-heavy" post, where we go into implementing automata for real and useful things. 

As always the programming language is Rust. By now I've actually had a bit of practice with the language, so hopefully the code will be less naive. Where in the [previous post on Finite Automata]({% post_url 2016-04-10-finite-automata %}) we went through examples of direct encodings of specific automata, in this post we'll look at more reusable code. I hope to publish the code discussed here in a crate eventually. 

This is part one out of <del>two</del> <ins>three</ins>. It's taking too long to write everything in one post, so I decided to split it up and publish this part first. The full code of this blogpost is [tagged on github](https://github.com/Apanatshka/dnfa/tree/blogpost-part-1). 

# Non-deterministic Finite Automata

Quick recap: The so-called NFA goes from state to state based on the input symbol. Once we're out of input symbols, if the state is a "final" state, we accept the input. The non-deterministic part means that from any state an input symbol can direct us to zero or more other states, so we can be in multiple states at once.

So let's look at a general framework for NFAs (don't panic, explanation below):

```rust
use std::collections::HashMap;
use std::collections::HashSet;

struct NFAHashState<Input, StateRef, Payload> {
    transitions: HashMap<Input, HashSet<StateRef>>,
    payload: Option<Payload>,
}

pub struct NFA<Input, Payload> {
    alphabet: Vec<Input>,
    states: Vec<NFAHashState<Input, usize, Payload>>,
}
```

I did say *general framework*, so generics galore :) Let's take it apart:

We have a state of an NFA which can take certain `Input`, uses `StateRef` to refer to other states, and has a `Payload`. The `Payload` refers to data that the automaton returns when it's in a final state. Inside the struct is the transition map from input to a set of state references, and the `Option<Payload>` where `None` means non-final state and `Some(payload)` means a final state with a `payload`. 

The `NFA` struct holds the states and the alphabet. It is also generic over `Input` and `Payload`. Maybe it should really be generic over the exact state struct rather than the `Payload`, but I'm not sure as that would require a `NFAState` trait... Whatever, we're going with this for now. 

## NFA execution

For the simplest interaction with an existing NFA, we just supply it some input and see if it "accepts" it. Let's look into that first:

```rust
pub const AUTO_START: StateNumber = 0;

impl<Input: Eq + Hash, Payload: Clone> NFA<Input, Payload> {
    pub fn apply<I: AsRef<[Input]>>(&self, input: I) -> Option<Payload> {
        let mut cur_states = HashSet::new();
        let mut nxt_states = HashSet::new();
        cur_states.insert(AUTO_START);
        for symbol in input.as_ref() {
            for &cur_state in &cur_states {
                if let Some(nxts) = self.states[cur_state].transitions.get(symbol) {
                    nxt_states.extend(nxts);
                }
            }
            cur_states.clear();
            mem::swap(&mut cur_states, &mut nxt_states);
        }
        cur_states.iter().filter_map(|&state| self.states[state].payload.clone()).next()
    }
}
```

The bounds on the generics are because we use `HashSet`s and maps. We also require `Payload: Clone` so we can give back an `Option<Payload>`. The `apply` method uses `AsRef` to be able to take `Vec<Input>` directly, or `&str` if `Input=u8`. 

The implementation creates two sets: current states and next states. We start in `AUTO_START`, a predefined (constant) start state. For every `symbol` in the `input` we go over the current states. We use the `symbol` and `cur_state` to find `nxts` (next states) and add them to the `nxt_states` set. After going through all current states we clear the `cur_states` and swap it with the `nxt_states`. So the `nxt_states` is empty again and the `cur_states` are filled for the next `symbol`. This `clear` and `swap` is slightly more memory efficient than doing `cur_states = nxt_states; nxt_states = HashSet::new();` because `clear` doesn't throw away the already allocated memory. Hurray for premature optimisation! Anyway, after all the input has been processed, we grab the first payload we can find. 

We're not focussing on performance here, but obviously building these sets in the inner loop is kind of terrible. Let's fix that, by turning the NFA into a DFA, a *Deterministic* Finite Automaton. Basically we'll need to pre-compute the sets of states you can be in and make those single states in the DFA. This mean we have an potential combinatorial explosion of states on our hands, which could make things worse.. Eh ¯\\\_(ツ)\_/¯

# Powerset Construction (NFA &rarr; DFA)

The standard algorithm for NFA to DFA transformation is powerset construction. It's named after the state-space of the resulting DFA, which is the powerset of the states of the NFA. The `DFA` struct and `DFAHashState` are very similar to the `NFA` one, so I'm not going to show them here to save a bit of space. Basically the only difference is that the `DFAHashState` transitions don't map to a `HashSet` but to a single `StateRef`. Ah, and here we see that we could make the `NFA` struct a more general struct that is generic over the `states` so we can reuse it for the `DFA`. Note to self: fix that up. 

```rust
impl<Input: Eq + Hash + Clone, Payload: Clone> NFA<Input, Payload> {
    // ...
    
    pub fn powerset_construction<F>(&self, payload_fold: &F) -> DFA<Input, Payload>
        where F: Fn(Option<Payload>, &Option<Payload>) -> Option<Payload>
    {
        type StateRef = usize;

        let mut states = vec![DFAHashState::new()];
        let mut states_map: HashMap<BTreeSet<StateRef>, StateRef> = HashMap::new();
        let cur_states: BTreeSet<StateRef> = iter::once(AUTO_START).collect();

        states[AUTO_START].payload = self.states[AUTO_START].payload.clone();
        states_map.insert(cur_states.clone(), AUTO_START);

        psc_rec_helper(self, &mut states, &mut states_map, cur_states, AUTO_START, payload_fold);

        DFA {
            alphabet: self.alphabet.clone(),
            states: states,
        }
    }
}
```

So first off, we need room for the `states` of the DFA. Then we need to be able to rename the sets of NFA states, so we create a `states_map` for that. We use a `BTreeSet<StateRef>` because `HashSet` doesn't implement the `Hash` trait itself. 

The current states don't actually need to be mutated after creation, so instead of creating an empty set and mutating it, we can build one from an iterator. In this case we have only one item to put in, so we can build an iterator for it with `iter::once`. (We could also write a macro like `vec!` instead, but.. meh, too much trouble[^1]). 

Our DFA starts with an `AUTO_START` state. The start state is the same for the NFA and DFA, so we need to copy the payload. Then we insert a ground truth into the `states_map`: being in *only* the NFA start state, means we're in the DFA start state. 

Now let's look at the `psc_rec_helper`, which stands for PowerSetConstruction-Recursion-Helper:

```rust
fn psc_rec_helper<Input, Payload, F>(nfa: &NFA<Input, Payload>,
                                     states: &mut Vec<DFAHashState<Input, usize, Payload>>,
                                     states_map: &mut HashMap<BTreeSet<usize>, usize>,
                                     cur_states: BTreeSet<usize>,
                                     cur_num: usize,
                                     payload_fold: &F)
    where Input: Eq + Hash + Clone,
          Payload: Clone,
          F: Fn(Option<Payload>, &Option<Payload>) -> Option<Payload>
{
    for symbol in &nfa.alphabet {
        let mut nxt_states = BTreeSet::new();
        let mut payload = None;
        for &cur_state in &cur_states {
            if let Some(states) = nfa.states[cur_state].transitions.get(symbol) {
                nxt_states.extend(states);
            }
        }

        if nxt_states.is_empty() {
            continue;
        }

        let nxt_num = states_map.get(&nxt_states).cloned().unwrap_or_else(|| {
            let nxt_num = states.len();
            let payload = nxt_states.iter()
                .map(|&st| &nfa.states[st].payload)
                .fold(None, payload_fold);
            states.push(DFAHashState::with_payload(payload));
            states_map.insert(nxt_states.clone(), nxt_num);
            psc_rec_helper(nfa, states, states_map, nxt_states, nxt_num, payload_fold);
            nxt_num
        });

        states[cur_num].transitions.insert(symbol.clone(), nxt_num);
    }
}
```

So we go over every symbol in the alphabet. And for each of them we collect the `nxt_states` that we can reach with that symbol from the `cur_states`. Then we look up the DFA state number for `nxt_states` in `states_map`. If we find one, we don't go into the `unwrap_or_else` and just record that transition from `cur_num` to `nxt_num` on the current symbol.  
If there isn't a DFA state number for `nxt_states` yet, then we haven't seen it before. We create new DFA states in the order that we find them, so the new `nxt_num` is the size of the states vector of `dnfa`. We should add the new state to that vector too, and record it's payload[^2]. I sneaked in a state constructor helper function thingy (how do you call this in Rust anyway?) called `with_payload`. That one is hopefully self-explanatory.  
Now that we've discovered a new state we didn't know about before, we'll also recursively call `psc_rec_helper` to record the transitions for it. And that's it. 

## Improving the code

Although I spent some time making the code in this post readable, I also found that some steps in between might be worth showing you. So here are some improvements. 

### A bit of code-reuse

Have you noticed how we keep repeating this loop that computes the next states? It's fairly easy to pull out, although it requires a bit of extra generics to make it fit for the `HashSet` in `apply` and the `BTreeSet` in `psc_rec_helper`. This is what it looks like:

```rust
impl<Input: Eq + Hash, Payload: Clone> NFA<Input, Payload> {
    #[inline]
    fn _next_state<'i, 'j, Iter, Ext>(&'j self, states: Iter, symbol: &Input, nxt_states: &mut Ext)
        where Iter: IntoIterator<Item = &'i usize>,
              Ext: Extend<&'j usize>
    {
        for &state in states {
            if let Some(states) = self.states[state].transitions.get(symbol) {
                nxt_states.extend(states);
            }
        }
    }
}
```

### Recursion to worklist

Doesn't that recursive helper function feel unsatifying? Does it really matter if we discover the transitions of the DFA in depth-first order, switching to new states asap? Not really. So let's change this recursion into a loop with our own stack of work:

```rust
impl NFA {
    // ...
    
    pub fn powerset_construction<F>(&self, payload_fold: &F) -> DFA<Input, Payload>
        where F: Fn(Option<Payload>, &Option<Payload>) -> Option<Payload>
    {
        type StateRef = usize;

        let mut states = vec![DFAHashState::new()];
        let mut states_map: HashMap<BTreeSet<StateRef>, StateRef> = HashMap::new();
        let cur_states: BTreeSet<StateRef> = iter::once(AUTO_START).collect();

        states[AUTO_START].payload = self.states[AUTO_START].payload.clone();
        states_map.insert(cur_states.clone(), AUTO_START);

        let mut worklist = vec![(cur_states, AUTO_START)];
        while let Some((cur_states, cur_num)) = worklist.pop() {
            for symbol in &self.alphabet {
                let mut nxt_states = BTreeSet::new();
                self._next_state(&cur_states, symbol, &mut nxt_states);

                // Skip the stuck state
                if nxt_states.is_empty() {
                    continue;
                }

                let nxt_num = states_map.get(&nxt_states).cloned().unwrap_or_else(|| {
                    let nxt_num = states.len();
                    let payload = nxt_states.iter()
                        .map(|&st| &self.states[st].payload)
                        .fold(None, payload_fold);
                    states.push(DFAHashState::from_payload(payload));
                    states_map.insert(nxt_states.clone(), nxt_num);
                    worklist.push((nxt_states, nxt_num));
                    nxt_num
                });

                states[cur_num]
                    .transitions
                    .insert(symbol.clone(), nxt_num);
            }
        }

        DFA {
            alphabet: self.alphabet.clone(),
            states: states,
        }
    }
}
```

Note that we now put our newly found states on a stack called `worklist` instead of doing a recursive call. This is done inside a while-loop that pops work off the `worklist` again. So as soon as we discover no new states, the stack will decrease and the loop will end. 

# A general interface

We're already using `AUTO_START` and `_next_state` for these automata. What else can be make them do similarly? I cheated and took a peek at the interface of the [`aho-corasick`](https://github.com/burntsushi/aho-corasick) [Automaton](https://github.com/BurntSushi/aho-corasick/blob/master/src/autiter.rs). The basic idea is: have an iterator of intermediate matches. So we don't just return the `payload` at the end of the input, but return matches whenever we hit a final state (i.e. with a payload). And so we get the following trait:

```rust
pub trait Automaton<Input, Payload> {
    type State: Debug;

    fn start_state() -> Self::State;

    fn next_state(&self, state: &Self::State, input: &Input) -> Self::State;

    fn get_match(&self, state: &Self::State, text_offset: usize) -> Option<Match<Payload>>;

    fn find<'i, 'a>(&'a self, s: &'i [Input]) -> Matches<'i, 'a, Input, Payload, Self>
        where Self: Sized
    {
        Matches {
            aut: self,
            input: s,
            offset: 0,
            state: Self::start_state(),
        }
    }
}
```

Why the hassle with a `State` and `start_state()` etc? Because `NFA`s are in a set of states at a time, and `DFA`s are in only one. Ok, so let's look into these `Match` and `Matches`:

```rust
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub struct Match<Payload> {
    pub payload: Payload,
    pub end: usize,
}

#[derive(Debug)]
pub struct Matches<'i, 'a, Input: 'i, Payload, A: 'a + Automaton<Input, Payload>> {
    aut: &'a A,
    input: &'i [Input],
    offset: usize,
    state: A::State,
}

impl<'i, 'a, Input, Payload, A: Automaton<Input, Payload>> Iterator
    for Matches<'i, 'a, Input, Payload, A> {
    type Item = Match<Payload>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut offset = self.offset;
        while offset < self.input.len() {
            self.state = self.aut.next_state(&self.state, &self.input[offset]);
            offset += 1;
            if let Some(m) = self.aut.get_match(&self.state, 0) {
                self.offset = offset;
                return Some(m);
            }
        }
        None
    }
}
```

So a `Match` consists of a `payload` and the offset into the text where the `end` of the "match" is. The `Matches` iterator keeps track of the `offset` and mostly leans on the trait functions to find the next match. So let's look at an implementation of the trait:

```rust
impl<Input: Eq + Hash, Payload: Clone> Automaton<Input, Payload> for NFA<Input, Payload> {
    type State = HashSet<usize>;

    #[inline]
    fn start_state() -> Self::State {
        iter::once(AUTO_START).collect()
    }

    #[inline]
    fn next_state(&self, states: &Self::State, symbol: &Input) -> Self::State {
        let mut nxt_states = HashSet::new();
        self._next_state(states, symbol, &mut nxt_states);
        nxt_states
    }

    #[inline]
    fn get_match(&self, states: &Self::State, text_offset: usize) -> Option<Match<Payload>> {
        for &state in states {
            if let Some(ref payload) = self.states[state].payload {
                return Some(Match {
                    payload: payload.clone(),
                    end: text_offset,
                });
            }
        }
        None
    }
}
```

That start state set should perhaps be put into a [`lazy_static`](https://docs.rs/crate/lazy_static/0.2.1), but that's yet another little thing I'll sacrifice to get this post published. Note how all the implemented functions are annotated with `#[inline]`, which will hopefully make the inner loop of the `Matches` iterator a bit faster. The `next_state` method uses are previously defined `_next_state` (now you get where that name came from ^^). `get_match` finds the first state in the set of states which has a payload. I'm not completely sure that this is the right approach, since the `DFA` version (through powerset construction) will give the combined payload of all states that the `NFA` would be in. On the other hand, it's not that easy to get the payload folding function in here. I may or may not change this implementation to return a list of payloads instead just the first one.. Hmm, whatever ¯\\\_(ツ)\_/¯

# Footnotes

[^1]: I mean sure, it's not hard to write a `vec!`-like macro for `HashSet`, but it's not super easy to reuse in multiple projects. And I'm not volunteering to create and maintain a std-lib-extra-utilities crate. Or.. huh, maybe that's not a bad idea. Though it might just be better to contribute it to the std library...
[^2]: If you're trying something like this with a folding function as parameter, make sure you get the type right. It took me quite a while to figure out how to make `fold` not eat my `FnMut`. As you can see, I got it working by using a borrow of a `Fn` instead (because `FnMut` is implemented for that). 
