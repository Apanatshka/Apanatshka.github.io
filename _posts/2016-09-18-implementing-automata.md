---
layout:   post
title:    "Implementing Automata"
date:     2016-09-18
category: CompSci
tags:     [theory, automata, computation, push-down automata, stack, context-free languages, context-free grammar, context-free]
---

This is post number three in a [series]({% post_url 2016-03-28-theory-of-computation %}) on Finite Automata. This is the promised "implementation-heavy" post, where we go into implementing automata for real and for useful things. 

As always the programming language is Rust, and this time I've actually had a bit of practice with the language. Where in the [previous post on Finite Automata]({% post_url 2016-04-10-finite-automata %}) we went through examples of direct encodings of specific automata, in this post we'll look at more reusable code. I hope to publish the code discussed here in a crate eventually. 

# Non-deterministic Finite Automata

Quick recap: The so-called NFA goes from state to state based on the input symbol, and once we're out of input if the state is a "final" state, we accept the input. The non-deterministic part means that from any state an input symbol can direct us to zero or more other states, so we can be in multiple states at once.

So let's look at a general framework for NFAs:

```rust
pub type Input = u8;
pub type StateNumber = usize;

#[derive(Clone, Default)]
struct NFAState {
    transitions: BTreeMap<Input, BTreeSet<StateNumber>>,
    is_final: bool,
}

#[derive(Default)]
pub struct NFA {
    alphabet: Vec<Input>,
    states: Vec<NFAState>,
}

impl NFAState {
    fn new() -> Self {
        NFAState {
            transitions: BTreeMap::new(),
            is_final: false,
        }
    }
}

impl NFA {
    pub fn new() -> Self {
        NFA {
            alphabet: Vec::new(),
            states: Vec::new(),
        }
    }
}
```

Don't worry, we'll look into optimisation and generalisation later. For now we have an NFA that works on bytes (`u8`) and can have at most `usize::MAX` states. The bytes line up well with ASCII characters, which works well enough for this example. 

An `NFAState` has transitions to sets of other states, and can be marked as final. We might generalise that final boolean to some kinds of payload that can be queried to see if it counts as final. The `NFA` itself records the entire alphabet, which will be useful later. It also contains the whole vector of `states`, and a `StateNumber` is an offset in this vector. 

## Testing an input

For the simplest interaction with an existing NFA, we just supply it some input and see if it "accepts" it. Let's look into that first:

```rust
pub const AUTO_START: StateNumber = 0;

impl NFA {
    // ...
    
    pub fn apply(&self, input: &[Input]) -> bool {
        let mut cur_states = BTreeSet::new();
        let mut nxt_states = BTreeSet::new();
        cur_states.insert(AUTO_START);
        for &byte in input {
            for cur_state in cur_states {
                if let Some(nxts) = self.states[cur_state].transitions.get(&byte) {
                    nxt_states.extend(nxts);
                }
            }
            cur_states = nxt_states;
            nxt_states = BTreeSet::new();
        }
        cur_states.iter().any(|&state| self.states[state].is_final)
    }
}
```

(For usability, we should really use `input: AsRef<[Input]>`, but I only just noticed, and I'm too lazy to change it right now). Let's see what we have here:

1. We take some input and will return a `bool`. As expected.
2. We create a set for the current states and for the next states.
3. We start at a predefined start state. 
4. For each byte we take a step.
    1. For each current state we try to follow the transition that corresponds to the byte from the input. If it exists the states from the transition are added to the next states. 
    2. After that we rename the next states the current states and reset the next states to empty. 
5. We check if any current state is a final state.

We're not focussing on performance here, but obviously building these sets (even if they were `HashSet`s) in the inner loop is kind of terrible. Let's fix that, by turning the NFA into a DFA, a *Deterministic* Finite Automaton. Basically we'll need to pre-compute there sets of states you can be in and make those single states in the DFA. This mean we have an potential explosion of states on our hands, which could make things worse.. But let's try it anyway. 

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
