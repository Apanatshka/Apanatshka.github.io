https://github.com/jaheba/stuff/issues/1

Perhaps it's interesting to add a little comparison to a different way to incorporate unit that I see described much less: Use physical quantity for the type instead of a single unit. The simplest way that works in almost any type system is to decide on a normalisation, so your type would be:

```rust
// Normalise to whole degrees celcius
#[derive(Debug, Clone, Copy)]
pub struct Temperature {
  degC: f64, // note how this field is private
};
```

And your construction/destruction (without thinking about traits right now) would be:

```rust
impl Temperature {
  pub fn from_celcius(degrees: f64) -> Temperature {
    return Temperature { degC = degrees };
  }
  pub fn from_fahrenheit(degrees: f64) -> Temperature {
    return Temperature { degC = (degrees - 32.) * 5./9. };
  }
  pub fn to_celcius(self) -> f64 {
    return self.degC;
  }
  pub fn to_fahrenheit(self) -> f64 {
    return Temperature { degC = degrees * 9./5. + 32. };
  }
}
```

Of course when you have algebraic data types (`enum` in Rust), you can also defer the normalisations like in the unit-as-type approach. This may not only save cpu cycles, but also accuracy when working with floating point units!
Since Rust has `From`/`Into`, I think the unit-as-type approach is still the best way because the compiler will more easily optimise `newtype` structs than `enum`s. But I just wanted to add the physical-quantity-as-type approach to complete things. 
