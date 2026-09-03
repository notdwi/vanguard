import { en, type TranslationKey } from './en'
import { ptBR } from './ptBR'

export type Language = 'en' | 'pt-BR'

export const languages: { id: Language; label: string }[] = [
  { id: 'en', label: 'English' },
  { id: 'pt-BR', label: 'Português (BR)' },
]

const dictionaries: Record<Language, Record<TranslationKey, string>> = {
  en,
  'pt-BR': ptBR,
}

export type Vars = Record<string, string | number>

export function translate(lang: Language, key: TranslationKey, vars?: Vars): string {
  const template = dictionaries[lang]?.[key] ?? en[key] ?? key
  if (!vars) return template
  return template.replace(/\{(\w+)\}/g, (match, name: string) =>
    name in vars ? String(vars[name]) : match,
  )
}

export type { TranslationKey }
