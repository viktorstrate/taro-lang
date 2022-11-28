import * as taro from '../../pkg/taro'

const codeTextarea =
  document.querySelector<HTMLTextAreaElement>('#code-textarea')!
const outputRegion = document.querySelector<HTMLDivElement>('#output-region')!

codeTextarea.addEventListener('input', (e) => {
  const value = (e.target as HTMLTextAreaElement).value
  outputRegion.innerText = taro.compile(value)
})

outputRegion.innerText = taro.compile(codeTextarea.value)

export {}
