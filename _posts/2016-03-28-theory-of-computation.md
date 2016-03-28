---
layout:   post
title:    "Theory of Computation"
date:     2016-03-28
category: CompSci
tags:     [announcement, theory, computation, automata, basics, rust]
---

# What

This is just an announcement post. I'm going to write some posts about theoretical computer science stuff. The planned 'stuff' is: finite automata, regular languages, and context-free languages. The plan is to work up to parsers and parsing algorithms eventually, but let's see how far I get and quickly I get there first.

# How

To start things off I'm going to use the book [Introduction to the Theory of Computation](https://books.google.nl/books?id=VJ1mQgAACAAJ) by Michael Sipser. I have the second edition, the international one. I picked this book because I was taught with this book, and it's pretty good in my opinion. Granted I've only read two books on this topic, but in comparison this one is pretty complete and nice to read. I'm going to use some of the ordering and examples from the book and check that I'm using the right terminology. But to make sure I don't rip it off, I'll re-explain things in my own words and add some extras (see [P.S.: Rust](#ps-rust)). 

# Why

I write the blog posts to practice writing. When I wrote my master's thesis I found out that writing *well* is pretty hard. I expect to need to write a lot more in the coming years, so I need to learn how to construct a narative, keep to it, and keep it interesting. So I'm going to start with some theory that underlies a topic that I like: parsers. And I'm going to write about it in the style that I prefer: informal. Hopefully that will also help me improve my formal writing skills. In the mean time I'll be writing about interesting theoretical subjects, so I get to practice making the posts interesting to read. I have my pet peeves about theoretical texts, so hopefully I can present the subject material in a way that I find nice. Which is mostly concrete examples, and not using single (greek) letters/symbols for everything. Let's see how it goes!

# P.S.: Rust

This stuff is very theorical, so to spice things up I'm going to try to add a lot of code snippets. For the programming language I picked [Rust](https://www.rust-lang.org/) because I'd like to learn the language, and this seems like a good opportunity to try it out. That does mean that I'll be figuring out the language as I write the posts, so I'll probably make mistakes. But whatever, that's how you learn :)
