import {
  SidebarGroup,
  SidebarGroupLabel,
  SidebarMenu,
  SidebarMenuItem,
  SidebarMenuButton,
} from "./sidebar";

const items = [
  "first chat",
  "second chat",
  "third chat",
  "fourth chat",
  "fifth chat",
  "sixth chat",
  "seventh chat",
  "eighth chat",
];

const ChatNav = () => {
  return (
    <SidebarGroup>
      <SidebarGroupLabel>Chats</SidebarGroupLabel>
      <SidebarMenu>
        {items.map((item) => (
          //TODO: Add liquid glass and change text color
          <SidebarMenuItem
            key={item}
            className="mb-4 rounded-3xl border border-white/20 bg-white/20 backdrop-blur-2xl shadow-[0_0_0_1px_rgba(255,255,255,0.12),0_18px_45px_rgba(15,23,42,0.65)]"
          >
            <SidebarMenuButton className="text-white">
              <span>{item}</span>
            </SidebarMenuButton>
          </SidebarMenuItem>
        ))}
      </SidebarMenu>
    </SidebarGroup>
  );
};

export default ChatNav;
