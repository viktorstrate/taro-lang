# Transpilation examples

## Struct

```

struct Test {
  let defaultVal = 123
  let mut noDefault: Bool
}

----

function Strut__Test(defaultVal, noDefault) {
  this.defaultVal = defaultVal ?? 123
  this.noDefault = noDefault
}

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

Object.assign(Strut__Test.protocol, {
  print() {
    console.log("Test struct: " + self.defaultVal)
  }
})

```

