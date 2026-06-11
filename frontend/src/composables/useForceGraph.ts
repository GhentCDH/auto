import { onBeforeUnmount, ref, shallowRef } from 'vue';
import {
  forceCenter,
  forceCollide,
  forceLink,
  forceManyBody,
  forceSimulation,
  type Simulation,
  type SimulationNodeDatum,
} from 'd3-force';
import type { GraphEdge, GraphNode } from '@/composables/useOrgGraph';

export interface SimNode extends SimulationNodeDatum, GraphNode {
  radius: number;
}

export interface SimEdge {
  source: SimNode;
  target: SimNode;
}

/** Node radius scales gently with connection count. */
function radiusFor(degree: number): number {
  return Math.min(10 + Math.sqrt(degree) * 4, 26);
}

export function useForceGraph(width: number, height: number) {
  const simNodes = shallowRef<SimNode[]>([]);
  const simEdges = shallowRef<SimEdge[]>([]);
  // Bumped every tick so dependent computed/templates re-read node positions.
  const tick = ref(0);

  let sim: Simulation<SimNode, undefined> | null = null;
  let w = width;
  let h = height;

  function setData(nodes: GraphNode[], edges: GraphEdge[]) {
    // Preserve positions of nodes that already exist across refreshes.
    const prev = new Map(simNodes.value.map((n) => [n.id, n]));
    const next: SimNode[] = nodes.map((n) => {
      const old = prev.get(n.id);
      return {
        ...n,
        radius: radiusFor(n.degree),
        x: old?.x,
        y: old?.y,
        vx: old?.vx,
        vy: old?.vy,
      };
    });
    const byId = new Map(next.map((n) => [n.id, n]));
    const links = edges
      .map((e) => ({ source: byId.get(e.source)!, target: byId.get(e.target)! }))
      .filter((e) => e.source && e.target);

    simNodes.value = next;
    simEdges.value = links;

    sim?.stop();
    sim = forceSimulation<SimNode>(next)
      .force(
        'link',
        forceLink<SimNode, SimEdge>(links)
          .id((d) => d.id)
          .distance(90)
          .strength(0.4)
      )
      .force('charge', forceManyBody().strength(-340))
      .force('center', forceCenter(w / 2, h / 2))
      .force('collide', forceCollide<SimNode>().radius((d) => d.radius + 8))
      .on('tick', () => {
        tick.value++;
      });
  }

  function resize(width: number, height: number) {
    w = width;
    h = height;
    sim?.force('center', forceCenter(w / 2, h / 2));
    sim?.alpha(0.3).restart();
  }

  function reheat() {
    sim?.alpha(0.6).restart();
  }

  function dragStart(node: SimNode) {
    sim?.alphaTarget(0.3).restart();
    node.fx = node.x;
    node.fy = node.y;
  }

  function drag(node: SimNode, x: number, y: number) {
    node.fx = x;
    node.fy = y;
  }

  function dragEnd(node: SimNode, pin: boolean) {
    sim?.alphaTarget(0);
    if (!pin) {
      node.fx = null;
      node.fy = null;
    }
  }

  onBeforeUnmount(() => sim?.stop());

  return {
    simNodes,
    simEdges,
    tick,
    setData,
    resize,
    reheat,
    dragStart,
    drag,
    dragEnd,
  };
}
