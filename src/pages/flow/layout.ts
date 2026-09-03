import type { FlowEdge, FlowGraph, FlowNode } from '@/types/analysis'

export const NODE_WIDTH = 232
export const NODE_HEIGHT = 52
export const COLUMN_GAP = 40
export const ROW_GAP = 96

export interface PlacedNode extends FlowNode {
  x: number
  y: number
}

export interface RoutedEdge extends FlowEdge {
  path: string
  labelX: number
  labelY: number
}

export interface Layout {
  nodes: PlacedNode[]
  edges: RoutedEdge[]
  width: number
  height: number
}

/// Places nodes in depth bands, then routes each edge as a vertical S-curve.
export function layoutGraph(graph: FlowGraph): Layout {
  const bands = new Map<number, FlowNode[]>()
  for (const node of graph.nodes) {
    const list = bands.get(node.depth) ?? []
    list.push(node)
    bands.set(node.depth, list)
  }

  const depths = [...bands.keys()].sort((a, b) => a - b)
  const widest = Math.max(1, ...depths.map((d) => bands.get(d)?.length ?? 0))
  const width = widest * NODE_WIDTH + (widest - 1) * COLUMN_GAP
  const placed = new Map<string, PlacedNode>()

  depths.forEach((depth, bandIndex) => {
    const row = bands.get(depth) ?? []
    const rowWidth = row.length * NODE_WIDTH + (row.length - 1) * COLUMN_GAP
    const startX = (width - rowWidth) / 2
    row.forEach((node, i) => {
      placed.set(node.id, {
        ...node,
        x: startX + i * (NODE_WIDTH + COLUMN_GAP),
        y: bandIndex * (NODE_HEIGHT + ROW_GAP),
      })
    })
  })

  const edges: RoutedEdge[] = []
  for (const edge of graph.edges) {
    const from = placed.get(edge.from)
    const to = placed.get(edge.to)
    if (!from || !to) continue

    const x1 = from.x + NODE_WIDTH / 2
    const y1 = from.y + NODE_HEIGHT
    const x2 = to.x + NODE_WIDTH / 2
    const y2 = to.y
    const mid = (y1 + y2) / 2

    edges.push({
      ...edge,
      path: `M ${x1} ${y1} C ${x1} ${mid}, ${x2} ${mid}, ${x2} ${y2}`,
      labelX: (x1 + x2) / 2,
      labelY: mid,
    })
  }

  const height = depths.length * NODE_HEIGHT + Math.max(0, depths.length - 1) * ROW_GAP

  return { nodes: [...placed.values()], edges, width, height }
}
