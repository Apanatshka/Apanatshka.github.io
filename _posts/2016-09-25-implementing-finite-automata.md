---
layout:   post
title:    "Implementing Finite Automata"
date:     2016-09-25
category: CompSci
tags:     [theory, automata, computation, push-down automata, stack, context-free languages, context-free grammar, context-free]
---

This is post number three in a [series]({% post_url 2016-03-28-theory-of-computation %}) on Finite Automata. This is the promised "implementation-heavy" post, where we go into implementing automata for real and useful things. 

As always the programming language is Rust. By now I've actually had a bit of practice with the language, so hopefully the code will be less naive. Where in the [previous post on Finite Automata]({% post_url 2016-04-10-finite-automata %}) we went through examples of direct encodings of specific automata, in this post we'll look at more reusable code. I hope to publish the code discussed here in a crate eventually. 

# Non-deterministic Finite Automata

Quick recap: The so-called NFA goes from state to state based on the input symbol. Once we're out of input symbols, if the state is a "final" state, we accept the input. The non-deterministic part means that from any state an input symbol can direct us to zero or more other states, so we can be in multiple states at once.

So let's look at a general framework for NFAs (don't panic, explanation below):

```rust
use std::collections::HashMap;
use std::collections::HashSet;
use std::hash::Hash;

#[derive(Clone)]
struct NFAHashState<Input: Eq + Hash, StateRef, Payload> {
    transitions: HashMap<Input, HashSet<StateRef>>,
    payload: Option<Payload>,
}

pub struct NFA<Input: Eq + Hash, Payload> {
    alphabet: Vec<Input>,
    states: Vec<NFAHashState<Input, usize, Payload>>,
}
```

I *did* say **general framework**. Generics galore. Let's take it apart:

We have a state of an NFA which can take certain `Input`, uses `StateNumber` to refer to other states, and has a `Payload`. I hope the first two are clear. The `Payload` refers to data that the automaton may return when it's a final state. Inside the struct is the transition map from input to a set of state references, and the `Option<Payload>` where `None` means non-final state and `Some(payload)` means a final state with a `payload`. As you can see, `HashMap` and `HashSet` are used, hence the `Input: Eq + Hash`. 

The `NFA` struct holds the states and the alphabet. It is also generic over `Input` and `Payload`. Maybe it should really be generic over the exact state struct rather than the `Payload`, but I'm not sure as that would require a `NFAState` trait.. Whatever, we're going with this for now. 

## NFA execution

For the simplest interaction with an existing NFA, we just supply it some input and see if it "accepts" it. Let's look into that first:

```rust
pub const AUTO_START: StateNumber = 0;

impl<Input: Eq + Hash, Payload: Clone> NFA<Input, Payload> {
    pub fn apply<I: AsRef<[Input]>>(&self, input: I) -> Option<Payload> {
        let mut cur_states = HashSet::new();
        let mut nxt_states = HashSet::new();
        cur_states.insert(AUTO_START);
        for ref byte in input.as_ref() {
            for &cur_state in &cur_states {
                if let Some(nxts) = self.states[cur_state].transitions.get(byte) {
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

The first generics should look familiar. We do require `Payload: Clone` so we can give back an `Option<Payload>`. The `apply` method uses `AsRef` to be able to take `Vec<Input>` directly, or `&str` if `Input = u8`. 

The implementation creates two sets: current states and next states. We start in `AUTO_START`, a predefined (constant) start state. For every `symbol` in the `input` we go over the current states. We use the `symbol` and `cur_state` to find `nxts` (next states) and add them to the `nxt_states` set. After going through all current states we clear the `cur_states` and swap it with the `nxt_states`. So the `nxt_states` is empty again and the `cur_states` are filled for the next `symbol`. This `clear` and `swap` is slightly more memory efficient than doing `cur_states = nxt_states; nxt_states = HashSet::new();` because `clear` doesn't throw away the already allocated memory. Anyway, after all the input has been processed, we grab the first payload we can find. 

We're not focussing on performance here, but obviously building these sets in the inner loop is kind of terrible. Let's fix that, by turning the NFA into a DFA, a *Deterministic* Finite Automaton. Basically we'll need to pre-compute there sets of states you can be in and make those single states in the DFA. This mean we have an potential explosion of states on our hands, which could make things worse.. But let's try it anyway. 

## Powerset Construction (NFA &rarr; DFA)

The standard algorithm for NFA to DFA transformation is powerset construction. It's named after the state-space of the resulting DFA, which is the powerset of the states of the NFA. Just so I don't have to introduce a `DFA` struct just yet, we're going to look at a powerset construction that will return an `NFA`, but this one will be deterministic. 

```rust
pub const AUTO_STUCK: StateNumber = 1;

impl NFA {
    // ...
    
    pub fn powerset_construction(&self) -> Self {
        let mut dnfa = NFA {
            alphabet: self.alphabet.clone(),
            states: vec![NFAState::new(); 2],
        };
        let mut states_map: HashMap<Vec<StateNumber>, StateNumber> = HashMap::new();
        let cur_states: BTreeSet<StateNumber> = iter::once(AUTO_START).collect();

        dnfa.states[AUTO_START].is_final = self.states[AUTO_START].is_final;

        states_map.insert(Vec::new(), AUTO_STUCK);
        states_map.insert(vec![AUTO_STUCK], AUTO_STUCK);
        states_map.insert(vec![AUTO_START], AUTO_START);
        
        psc_rec_helper(self, &mut dnfa, &mut states_map, cur_states, AUTO_START);
        dnfa
    }
}
```

So first off, we need the struct where we build the DFA, which is still of the `NFA` type, so we call it `dnfa` for confusion and giggles. Then we need to be able to rename the sets of NFA states, so we create a map for that. Just to highlight what might be annoying with `HashMap`s and sets, we use a `HashMap` here, which immediately means we can't use a `BTreeSet<StateNumber>` anymore. So we'll have to turn the set into a vector whenever we look things up in the `states_map`. Good thing we're not focussing on performance here.

The current states don't actually need to be mutated after creation, so instead of creating an empty set and mutating it, we can build one from an iterator. In this case we have only one item to put in, so we can build an iterator for it with `iter::once`. 

The `dnfa` starts with two states, the `AUTO_START` and `AUTO_STUCK` state. `AUTO_STUCK` is never final, you just land and stay there. The start state is the same for the NFA and DFA, so we need to copy the finality information of it. 

Then we insert some ground information into the `states_map`. If we follow some input in the NFA which leads to zero states, we're stuck. If we land in the NFA's stuck state (and only the stuck state) we're also stuck. And being in only the NFA start state, means we're in the DFA start state. 

Now let's look at the `psc_rec_helper`, which stands for PowerSetConstruction-Recursion-Helper:

```rust
fn psc_rec_helper(nfa: &NFA,
                  dnfa: &mut NFA,
                  states_map: &mut HashMap<Vec<StateNumber>, StateNumber>,
                  cur_states: BTreeSet<StateNumber>,
                  cur_num: StateNumber) {
    for &input in &dnfa.alphabet {
        let mut nxt_states = BTreeSet::new();
        let mut fin = false;
        for &cur_state in &cur_states {
            if let Some(states) = self.states[cur_state].transitions.get(&input) {
                nxt_states.extend(states);
                fin |= states.iter().any(|&st| nfa.states[st].is_final);
            }
        }
        let nxt_states_vec: Vec<StateNumber> = nxt_states.iter().cloned().collect();

        let nxt_num = {
            let dnfa_states = &mut dnfa.states;
            states_map.get(&nxt_states_vec).cloned().unwrap_or_else(|| {
                let nxt_num = dnfa_states.len();
                let mut new_state = NFAState::new();
                new_state.is_final = fin;
                dnfa_states.push(new_state);
                states_map.insert(nxt_states_vec, nxt_num);
                psc_rec_helper(nfa, dnfa, states_map, nxt_states, nxt_num);
                nxt_num
            })
        };

        dnfa.states[cur_num]
            .transitions
            .entry(input)
            .or_insert_with(BTreeSet::new)
            .insert(nxt_num);
    }
}
```

So we go over every symbol in the alphabet. And for each of them we collect the `nxt_states` that we can reach with that symbol from the `cur_states`. Then we look up the DFA state number for `nxt_states` in `states_map`. If we find one then we just record that transition from `cur_num` to `nxt_num` on the current symbol. If there isn't a DFA state number for `nxt_states` yet, then we haven't seen it before. We just create new DFA states in order that we find them, so the new `nxt_num` is the size of the states vector of `dnfa`. We should add the new state to that vector too, and record it's finality. That finality is, just as in `apply` earlier, whether any of the `nxt_states` is final. 

Now that we've discovered a new state we didn't know about before, we'll also recursively call `psc_rec_helper` to record the transitions for it. And that's it. 

Doesn't that recursive helper function feel unsatifying? Does it really matter if we discover the transitions of the DFA in depth-first order, switching to new states asap? Not really. So let's change this recursion into a loop with our own stack of work:

```rust
impl NFA {
    // ...
    
    pub fn powerset_construction(&self) -> Self {
        // Same initialisation stuff
        let mut dnfa = NFA {
            alphabet: self.alphabet.clone(),
            states: vec![NFAState::new(); 2],
        };
        let mut states_map: HashMap<Vec<StateNumber>, StateNumber> = HashMap::new();
        let cur_states: BTreeSet<StateNumber> = iter::once(AUTO_START).collect();

        dnfa.states[AUTO_START].is_final = self.states[AUTO_START].is_final;

        states_map.insert(Vec::new(), AUTO_STUCK);
        states_map.insert(vec![AUTO_STUCK], AUTO_STUCK);
        states_map.insert(vec![AUTO_START], AUTO_START);
        
        // NEW:
        let mut worklist = vec![(cur_states, AUTO_START)];
        while let Some((cur_states, cur_num)) = worklist.pop() {
            for &input in &dnfa.alphabet {
                let mut nxt_states = BTreeSet::new();
                let mut fin = BTreeSet::new();
                for &cur_state in &cur_states {
                    if let Some(states) = self.states[cur_state].transitions.get(&input) {
                        nxt_states.extend(states);
                        fin |= states.iter().any(|&st| nfa.states[st].is_final);
                    }
                }
                let nxt_states_vec: Vec<StateNumber> =
                    nxt_states.iter().cloned().collect();

                let nxt_num = {
                    let dnfa_states = &mut dnfa.states;
                    states_map.get(&nxt_states_vec).cloned().unwrap_or_else(|| {
                        let nxt_num = dnfa_states.len();
                        let mut new_state = NFAState::new();
                        new_state.is_final = fin;
                        dnfa_states.push(new_state);
                        states_map.insert(nxt_states_vec, nxt_num);
                        // Push to stack instead of recursion!
                        worklist.push((nxt_states, nxt_num));
                        nxt_num
                    })
                };

                dnfa.states[cur_num]
                    .transitions
                    .entry(input)
                    .or_insert_with(BTreeSet::new)
                    .insert(nxt_num);
            }
        }
        
        dnfa
    }
}
```

Note that we now put our newly found states on a stack called `worklist` instead of doing a recursive called. This is done inside a while-loop that pops work off the `worklist` again. So as soon as we discover no new states, the list will decrease and the loop will end. 
