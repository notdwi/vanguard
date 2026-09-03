import { Button, IconButton, Input } from '@/components/ui'

import css from './repeater.module.css'

export interface Pair {
  name: string
  value: string
}

export function PairEditor({
  pairs,
  onChange,
  addLabel,
  namePlaceholder,
  valuePlaceholder,
}: {
  pairs: Pair[]
  onChange: (next: Pair[]) => void
  addLabel: string
  namePlaceholder?: string
  valuePlaceholder?: string
}) {
  const update = (index: number, patch: Partial<Pair>) => {
    onChange(pairs.map((p, i) => (i === index ? { ...p, ...patch } : p)))
  }

  return (
    <div>
      {pairs.map((pair, index) => (
        <div className={css.pairRow} key={index}>
          <Input
            mono
            value={pair.name}
            placeholder={namePlaceholder}
            onChange={(e) => update(index, { name: e.target.value })}
          />
          <Input
            mono
            value={pair.value}
            placeholder={valuePlaceholder}
            onChange={(e) => update(index, { value: e.target.value })}
          />
          <IconButton
            icon="close"
            size={12}
            label="Remove"
            onClick={() => onChange(pairs.filter((_, i) => i !== index))}
          />
        </div>
      ))}
      <Button
        small
        icon="plus"
        onClick={() => onChange([...pairs, { name: '', value: '' }])}
      >
        {addLabel}
      </Button>
    </div>
  )
}
