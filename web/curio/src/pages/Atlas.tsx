import {
  addEdge,
  Background,
  Controls,
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

import { AppSidebar } from "@/components/app-sidebar";
import {
  SidebarInset,
  SidebarProvider,
  SidebarTrigger,
} from "@/components/ui/sidebar";

const initialNodes: Node[] = [
  {
    id: "curio",
    type: "input",
    position: { x: 80, y: 120 },
    data: { label: "Curio" },
  },
  {
    id: "vault",
    position: { x: 360, y: 40 },
    data: { label: "Vault" },
  },
  {
    id: "chat",
    position: { x: 360, y: 200 },
    data: { label: "Chat" },
  },
  {
    id: "insight",
    type: "output",
    position: { x: 640, y: 120 },
    data: { label: "Insight" },
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
      fitView
    >
      <Background />
      <Controls />
    </ReactFlow>
  );
}

const Atlas = () => {
  return (
    <SidebarProvider className="[--sidebar:var(--background)] [--sidebar-accent:var(--background)] [--sidebar-border:var(--border)]">
      <AppSidebar />
      <SidebarInset className="bg-background">
        <div className="flex h-screen w-full flex-col bg-background">
          <SidebarTrigger className="m-2 shrink-0" />
          <main className="min-h-0 flex-1 px-4 pb-4">
            <div className="h-full w-full overflow-hidden rounded-lg border border-border bg-background">
              <ReactFlowProvider>
                <AtlasCanvas />
              </ReactFlowProvider>
            </div>
          </main>
        </div>
      </SidebarInset>
    </SidebarProvider>
  );
};

export default Atlas;
