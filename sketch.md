
# Ideas

- Type safe
- Advanced generics
- Great interop with javascript
- Const by default
- Value types by default
- consteval extension blocks, eg. parse GraphQL to const code without macros

## JS Interop

```
let val: Value = @{ jsFunc() }
let a: Any = @{ { key: 'value', foo: 23 } }

let array: Array<Number> = @{ [1, 2, 3] }

trait Any {}

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
  static func == (lhs: Self, rhs: Self) -> Boolean {
    !(lhs != rhs)
  }

  static func != (lhs: Self, rhs: Self) -> Boolean {
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
  static func == (lhs: Self, rhs: Self) -> Boolean {
    lhs.x == rhs.x && lhs.y == rhs.y
  }
}

let a: Point<Number> = Point { x: 2, y: 5 }
let b = a // copy

```

## Primitives

```
struct Float {}
struct Integer {}
struct Bool {}

type Structural = struct { let a: Boolean }
struct { let a: Boolean }
```

## Tuples

```
let a: () = ()
let b: (Bool, Bool) = (true, false)
```

## Functions

```
func test(a: Number) {}

let sum = <T: Combinable>(a: T, b: T) -> Number { a + b }

let funcs = {
  sum: <T: Combinable>(a: T, b: T) -> Number { a + b },
  hello: (){ console.log('hello world') },
}
```

## Async functions

```
let fetch_data: (String) -> Promise<String> = async (url) {
  let res = await fetch(url)
  let text = await res.text()
  return text
}

let fetch_append = async (url: String) => (await fetch_data(url)) + "!"
```

## String interpolation

```
trait StringExpressible {
  init(string: String)
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