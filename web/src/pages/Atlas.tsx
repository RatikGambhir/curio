import {
  addEdge,
  Background,
  BackgroundVariant,
  Controls,
  Position,
  ReactFlow,
  ReactFlowProvider,
  useEdgesState,
  useNodesState,
  type Connection,
  type Edge,
  type Node,
} from "@xyflow/react";
import "@xyflow/react/dist/style.css";
import { useCallback } from "react";

import { PageHeader } from "@/components/page-header";

/* Edges run left to right, so every node takes input on its left edge and
   gives output on its right; the default top/bottom handles made horizontal
   links loop back on themselves. */
const horizontal = {
  sourcePosition: Position.Right,
  targetPosition: Position.Left,
} as const;

const initialNodes: Node[] = [
  {
    id: "curio",
    type: "input",
    position: { x: 0, y: 120 },
    data: { label: "Curio" },
    ...horizontal,
  },
  {
    id: "vault",
    position: { x: 300, y: 30 },
    data: { label: "Vault" },
    ...horizontal,
  },
  {
    id: "chat",
    position: { x: 300, y: 210 },
    data: { label: "Chat" },
    ...horizontal,
  },
  {
    id: "insight",
    type: "output",
    position: { x: 600, y: 120 },
    data: { label: "Insight" },
    ...horizontal,
  },
];

const initialEdges: Edge[] = [
  { id: "curio-vault", source: "curio", target: "vault" },
  { id: "curio-chat", source: "curio", target: "chat" },
  { id: "vault-insight", source: "vault", target: "insight" },
  { id: "chat-insight", source: "chat", target: "insight" },
];

function AtlasCanvas() {
  const [nodes, , onNodesChange] = useNodesState(initialNodes);
  const [edges, setEdges, onEdgesChange] = useEdgesState(initialEdges);

  const onConnect = useCallback(
    (connection: Connection) => {
      setEdges((currentEdges) => addEdge(connection, currentEdges));
    },
    [setEdges],
  );

  return (
    <ReactFlow
      nodes={nodes}
      edges={edges}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onConnect={onConnect}
      defaultEdgeOptions={{ type: "smoothstep" }}
      fitView
      fitViewOptions={{ padding: 0.35 }}
    >
      <Background variant={BackgroundVariant.Dots} gap={22} size={1.25} />
      <Controls showInteractive={false} position="bottom-right" />
    </ReactFlow>
  );
}

const Atlas = () => {
  return (
    <>
      <PageHeader title="Atlas" meta="Starter map · not saved" />
      <div className="relative min-h-0 flex-1">
        <ReactFlowProvider>
          <AtlasCanvas />
        </ReactFlowProvider>
        <p className="pointer-events-none absolute left-5 top-4 max-w-xs text-sm text-muted-foreground">
          Drag between handles to connect ideas. Changes last until you leave
          the page.
        </p>
      </div>
    </>
  );
};

export default Atlas;
