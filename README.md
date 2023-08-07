# The Taro Programming Language

A programming language that transpiles to pure javascript,
which aims for great interoperability with javascript, strictly typed, generics, trait oriented.
It is heavily inspired by many functional and modern languages like Swift, Rust and Haskell.

It is designed such that transpilation to javascript is as direct as possible, by leveraging javascript features with minimal overhead.

## Taro compared to Typescript

The main goal for both Taro and Typescript is to make programming javascript more safe.
Typescript does this by adding a type system on top of javascript.
Taro on the other hand is a completely different programming language that transpiles to javascript.
This allows for an even stricter and more flexible type system by leveraging [algebraic data types](https://en.wikipedia.org/wiki/Algebraic_data_type).
By making a completely new language, inconsistencies and complexities in Javascript can be removed and new features like [pattern matching](https://en.wikipedia.org/wiki/Pattern_matching) can be added.

## Features

- [x] Super fast type-checking and transpilation
- [x] Javascript interoperability
- [x] Strict type safety
- [x] Type inference
- [x] Structures
- [x] Tuples
- [x] Functions
- [x] Enumerations
- [ ] Traits
- [ ] Generics
- [ ] Extensions
- [ ] Async support
- [ ] Pattern matching
- [ ] Error handling
- [ ] Generate Typescript definitions

## Documentation

### Variables

Variables are declared using the `let` keyword.
By default a variable is immutable, to declare a mutable variable, the `var` keyword is used instead.

```
let greeting = "Hello, World!"
```

A type signature can optionally be added, either for clarity or in special cases where the compiler is unable to infer the type by itself.

```
let greeting: String = "Hello, World!"
```

### Structures

Structures are backed by javascript classes.
To declare a new structure, each of its attributes must be declared upfront.
Each attribute is declared in the same way as stand-alone variables are,
except that the value is optional.
When specified, this will be used as a default value when initialized.

```
struct Car {
  let maxSpeed: Number
  let model: String
  let wheels = 4
}
```

Next, the Car struct can be initialized and assigned to a variable.
Notice that the wheels attribute is not specified and the default value of 4 is used instead.
All attributes without a default value must be specified when instantiating a new instance.

```
let deLorean = Car { maxSpeed: 100, model: "DMC DeLorean" }
```

Attributes can then be retrieved as such.

```
let model: String = deLorean.model
```

### Tuples

Tuples are typed primitives that combine multiple expressions into one.

```
let a: (String, Number, Boolean) = ("hello", 42, true)
```

Each component can be accessed in the following way

```
let first: String = a.0
let second: Number = a.1
let third: Boolean = a.2
```

### Functions

Functions can either be declared normally,

```
func sum(a: Number, b: Number) -> Number {
  return a + b
}
```

or as a lambda.

```
let sum = (a: Number, b: Number) -> Number {
  return a + b
}
```

### Enumeration

An enum is declared with a name and then a list of values the enum can take.
Each value can have a tuple of data associated with it.

Enum declarations are only used to type-check and will be removed when compiled to javascript.

```
enum IP {
  v4(Number, Number, Number, Number)
  v6(String)
}

let my_ip = IP.v4(127, 0, 0, 1)
```

In contexts where the type can be inferred, it is not necessary to write the name of the enum.
This means that the last line above can also be written as such.

```
let my_ip: IP = .v4(127, 0, 0, 1)
```

### Javascript interoperability

Raw javascript can be inserted as an expression anywhere using an escape block.

The content of an escape block is not evaluated or checked by the compiler,
it will instead simply be inserted into the raw javascript output.

```
let num = @{ 1 + 2 }
```

This code will not compile however, this is because the compiler cannot infer the type of the expression `@{ 1 + 2 }`.
This can be solved by explicitly stating the type signature of the variable.

```
let num: Number = @{ 1 + 2 }
```

#### Exposing external Javascript

The `external` keyword can be added in front of a variable to indicate that the object exists somewhere else.

```
external add: (Number, Number) -> Number
let x = add(1, 2)
```