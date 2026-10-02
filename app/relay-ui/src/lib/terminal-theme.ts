export interface CssPropertyReader {
  getPropertyValue(name: string): string
}

export interface TerminalTheme {
  background: string
  foreground: string
  cursor: string
}

export function readTerminalTheme(styles: CssPropertyReader): TerminalTheme {
  return {
    background: styles.getPropertyValue('--bg-deep').trim(),
    foreground: styles.getPropertyValue('--ink').trim(),
    cursor: styles.getPropertyValue('--green').trim(),
  }
}
