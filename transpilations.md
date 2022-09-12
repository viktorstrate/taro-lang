# Transpilation examples

Examples of how Taro concepts are transpiled to javascript

## Structures

```
struct Test {
  let defaultVal = 123
  var noDefault: Bool
}

let val = Test { noDefault: false }

----

function Test(defaultVal, noDefault) {
  this.defaultVal = defaultVal ?? 123
  this.noDefault = noDefault
}

const val = new Test(null, false)
```

## Trait

```
trait Printable {
  func print()
}

extend Test: Printable {
  func print() {
    console.log("Test struct: " + self.defaultVal)
  }
}

----

Object.assign(Test.protocol, {
  print() {
    console.log("Test struct: " + self.defaultVal)
  }
})
```

## Enum

Enum definitions will only be used for type definitions and will be compiled away

```
enum IPAddress {
  v4(Number, Number, Number, Number)
  v6(String)
}

let ipValue: IPAddress = .v4(192, 168, 0, 1)

----

const ipValue = [0, [192, 168, 0, 1]]
```

## Pattern matching

```
let result = match ipValue {
  .v4(a, b, c, d) => {
    let inner = 10 * b
    a + b + c + d + inner
  }
  .v6(_) => return "error"
}

----

let result = null
switch (ipValue[0]) {
  case 1:
    const inner = 10 * ipValue[2]
    result = ipValue[1] + ipValue[2] + ipValue[3] + ipValue[4]
    break
  case 2:
    return "error"
}
```

```
if let .v4(a, b, c, d) = ipValue {
  return a + b + c + d
}

---

if ipValue[0] === 0 {
  return ipValue[1] + ipValue[2] + ipValue[3] + ipValue[4]
}
```

```
let .v4(a, b, c, d) = ipValue else {
  return "error"
}
let sum = a + b + c + d

----

if ipValue[0] !== 0 {
  return "error"
}
const sum = ipValue[1] + ipValue[2] + ipValue[3] + ipValue[4]
```