import * as taro from '../../pkg/taro'

import { basicSetup } from 'codemirror'
import { EditorView } from '@codemirror/view'

const codeArea = document.querySelector<HTMLTextAreaElement>('#code-area')!
const outputRegion = document.querySelector<HTMLDivElement>('#output-region')!

const compileOnChange = EditorView.updateListener.of((e) => {
  if (e.docChanged) {
    const value = e.state.doc.toJSON().join('\n')
    outputRegion.innerText = taro.compile(value)
  }
})

const code = `// declaring variables
let greeting = "Hello, World!"

// structures
struct Car {
  let maxSpeed: Number
  let model: String
  let wheels = 4
}

let deLorean = Car { maxSpeed: 100, model: "DMC DeLorean" }
let model: String = deLorean.model

// tuples
let a: (String, Number, Boolean) = ("hello", 42, true)
let first: String = a.0

// functions
func sum(a: Number, b: Number) -> Number {
  return @{ a + b }
}

// enumeration
enum IP {
  v4(Number, Number, Number, Number)
  v6(String)
}

let my_ip: IP = .v4(127, 0, 0, 1)

// javascript interoperability
let num: String = @{ 1 + 2 }
`

outputRegion.innerText = taro.compile(code)

new EditorView({
  extensions: [basicSetup, compileOnChange],
  parent: codeArea,
  doc: code,
})

// codeTextarea.addEventListener('input', (e) => {
//   const value = (e.target as HTMLTextAreaElement).value
//   outputRegion.innerText = taro.compile(value)
// })

// outputRegion.innerText = taro.compile(codeTextarea.value)

export {}
