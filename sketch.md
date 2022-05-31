
# Ideas

- Type safe
- Advanced generics
- Great interop with javascript
- Const by default
- Value types by default
- consteval extension blocks, eg. parse GraphQL to const code without macros

## JS Interop

```
let val: Any = @jsFunc()
let a: Any = @{ key: 'value', foo: 23 }

let array: [number] = unsafe val.unchecked_cast()

let result = @( externalCall() )
let isSafe = @( typeof result == 'object' && typeof result.key == 'string' )
if isSafe {
  // do cast
}

trait Any {
  unsafe func<T> unchecked_cast() -> T
}

```

## Types

```
// Type deduce to 'string'
let val = "hello"

// error, val is constant
val = "asdf"

// ok
let mut val2 = "foo"
val2 = "bar"

trait Equatable {
  static func == (lhs: Self, rhs: Self) -> boolean

  static func != (lhs: Self, rhs: Self) -> boolean {
    !(lhs == rhs)
  }
}

struct Point<T> {
  let x: T
  let y: T
}

extend Point {
  init() {
    self.x = 0
    self.y = 0
  }

  init(x: T, y: T) {
    self.x = y
    self.y = y
  }
}

extend Point: Equatable where T: Equatable {
  static func == (lhs: Self, rhs: Self) -> boolean {
    lhs.x == rhs.x && lhs.y == rhs.y
  }
}

let a: Point<number> = Point(x: 2, y: 5)
let b = a // copy

// Make aa a reference to a Point
let aa: &Point<number> = &Point { x: 2, y: 3 }
let bb = aa // not a copy
```

## Primitives

```
struct Float {}
struct Integer {}
struct Bool {}
```

## Tuples

```
let a: () = ()
let b: (Bool, Bool) = (true, false)
```

## Functions

```
func test(a: number) {
  console.log(`hello ${a}`)
}

let a: (number) -> () = (a: number) => {}
```

## String interpolation

```
trait StringExpressible {
  init(string: string)
}

trait StringInterpolationBaseExpressible: StringExpressible {
  mut func appendInterpolation(value: String)
}

trait StringInterpolationExpressible<T>: StringInterpolationBaseExpressible {
  mut func appendInterpolation(value: T)
}

trait Combinable {
  func ++(lhs: Self, rhs: Self) -> Self
}

extend<T, I> T: Combinable where T: StringInterpolationExpressible<I> {
  func ++(lhs: Self, rhs: Self) -> Self {
    let mut result = Self(lhs)
    result.appendInterpolation(rhs)
    return result
  }
}

```

## Extensions

```


trait Sequence: Indexable {
  associatedtype Element

  func index(at: Integer) -> Element

  func length() -> Integer
}

extend Array: Sequence {

}

```