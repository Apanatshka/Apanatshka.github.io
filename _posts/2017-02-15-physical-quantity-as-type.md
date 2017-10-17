---
layout:   post
title:    "Types, units and quantities"
date:     2017-02-15
category: CompSci
tags:     [units, physical quantity, Rust, programming, patterns, types]
---

*edited on 2017-02-16*

In this post I'd like to shortly discuss an idea I've had a long time ago about type systems and 
units of measure. The usual pitch about having units in the type system of a programming language 
starts with a sad story about some space craft crash because different teams used different 
measures of distance. The competing systems are usually 
[Imperial](https://en.wikipedia.org/wiki/Imperial_units) vs <del>Rebels</del> 
[<ins>Metric</ins>](https://en.wikipedia.org/wiki/Metric_system). Then units in the type system are 
introduced, which is a way to check that only numbers of the exact same unit are used in 
arithmetic[^arithmetic]. Examples of languages 
with first-class support are 
[F#](https://blogs.msdn.microsoft.com/andrewkennedy/2008/08/29/units-of-measure-in-f-part-one-introducing-units/) 
and [Fortress](https://blogs.oracle.com/projectfortress/entry/fortress_wrapping_up). 

[^arithmetic]: Addition anyway, I suppose multiplication should always work but just give you a different unit in return. 

I got triggered by a recent explanation of how to emulate [units of measure as types in 
Rust](https://github.com/jaheba/stuff/blob/master/communicating_intent.md). This can be done and is 
done in many languages (I think.. citation needed). But if you like Rust I can recommend reading 
the post, because it uses the Rust-specific conversion traits to its advantage as well. So you can 
work generically with the physical quantity *Temperature*, while the types are actually units of 
measure *Fahrenheit* and *Degrees Celsius*. 

What I find interesting is that all the successful systems I read about focus on units of measure.
The idea I'd like to explain in this post is unlikely to be original but I haven't the heart to look
up how much there is written about it. Gah, I'm beating around the bush. Let's just dive in.

> **EDIT:** Please note that although I was triggered by a Rust-specific post and I'm using Rust
> code below, this idea is expressly not based on Rust-specific features. I'm trying to explain it
> in a way that would work for almost any programming language. 

# The big idea

**Use *physical quantity* (e.g. Temperature) as the type instead of a specific *unit of measure* 
(e.g. centigrade)**.

There's two ways I can easily think of for implementing this. 

## Normalisation

The simplest way, that works in almost any type system, is to decide on a normalisation. There are a 
number of problems with this when you do scientific computation[^normalisation-problems], but I'm 
ignoring those for a second. Let's just look at some code that does this for temperature. I'll stay
with Rust as the implementation language:

[^normalisation-problems]: Like when you're measuring star distances in light-years but the Distance quantity is normalised to meters. And there are other issues with rounding errors. For most systems you can probably use a sufficiently large floating point value though, like, I don't know, a [128-bit floating point](https://en.wikipedia.org/wiki/Quadruple-precision_floating-point_format)? 

```rust
// Normalise to whole degrees celsius
#[derive(Debug, Clone, Copy)]
pub struct Temperature {
  degC: f64, // note how this field is private
}
```

Now to introduce units of measure, you have "smart constructors". These could be:

```rust
impl Temperature {
  pub fn from_celsius(degrees: f64) -> Temperature {
    Temperature { degC = degrees }
  }
  pub fn from_fahrenheit(degrees: f64) -> Temperature {
    Temperature { degC = (degrees - 32.) * 5./9. }
  }
  pub fn to_celsius(self) -> f64 {
    self.degC
  }
  pub fn to_fahrenheit(self) -> f64 {
    self.degC * 9./5. + 32.
  }
}
```

In Rust you would use these as `Temperature::from_celsius(20_f64)`. And then you could go nuts with
a compiler plugin to add special syntax that looks more like `20 C`. Or something slightly better
and more generic that would work with any units. 

## Late conversion

I already alluded to some issues with normalisation[^normalisation-problems]. When you have some 
form of case distinction (algebraic data types is what you think of of course, not some silly 
sub-typing hierarchy like OOP) in you type system (`enum` in Rust), you can also defer the 
normalisations. You can defer conversion entirely within generic calculations, which seems slightly 
more powerful than the units-as-types approach from the other post. Here's what it might look like:

```rust
// Normalise to whole degrees celsius
#[derive(Debug, Clone, Copy)]
enum Temperature {
  DegreesCelsius(f64),
  Fahrenheit(f64),
}

impl Temperature {
  pub fn from_celsius(degC: f64) -> Temperature {
    DegreesCelsius(degC)
  }
  pub fn from_fahrenheit(fahr: f64) -> Temperature {
    Fahrenheit(fahr)
  }
  pub fn to_celsius(self) -> f64 {
    match self {
      DegreesCelsius(degC) => degC,
      Fahrenheit(fahr) => (fahr - 32.) * 5./9.,
    }
  }
  pub fn to_fahrenheit(self) -> f64 {
    match self {
      DegreesCelsius(degC) => degC * 9./5. + 32.,
      Fahrenheit(fahr) => fahr,
    }
  }
}

/// Implementing addition on temperatures <3
impl Add for Temperature {
  type Output = Temperature;
  
  fn add(self, rhs: Temperature) -> Temperature {
    match (self, rhs) {
      (DegreesCelsius(l), DegreesCelsius(r)) => DegreesCelsius(l + r),
      (Fahrenheit(l), Fahrenheit(r)) => Fahrenheit(l + r),
      (DegreesCelsius(l), _) => DegreesCelsius(l + rhs.to_celsius()),
      (_, DegreesCelsius(r)) => DegreesCelsius(self.to_celsius() + r),
    }
  }
}
```

I think a primary downside of this scheme vs the units-as-types is that it's less 
extensible[^overhead]. An yet, however much I like extensibility, I think about it like this: If 
you want units you probably just want a crate (Rust equivalent of a library) that offers you 
everything you could possibly need. That takes a bit of time, but if everyone just contributes to 
the one crate, you should be able to collect everything you need[^precision]. It could be that 
simple. Unless I'm overlooking something? Eh, whatever ¯\\\_(ツ)\_/¯

> **EDIT**: But what about all the other features in the Rust type system? What about type
> parameters and traits and macros and, heck, why not even compiler plugins. Well.. that another
> thing you'll need to figure out on a per language basis. I suggesting keeping a look out for the
> release of the [uom crate](https://github.com/iliekturtles/uom), which is iteratively improving
> a units of measure implementation, based on quantities and normalisation actually :)

[^overhead]: Or perhaps the low-level devs mostly care about the memory overhead of the enums? Or maybe even about the branching in the code during calculations? In that case you should go with the normalising approach I guess. 
[^precision]: I guess there is the issue of control over precision, which languages with first-class units have better since they can (I think) have any type + unit combination. Maybe we can do something with type parameters.. Hmm, something to ponder/try. 
