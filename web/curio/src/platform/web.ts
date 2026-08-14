import { streamCurioChat } from "@/features/chat/chat-stream"
import {
  parseExternalHttpUrl,
  type PlatformServices,
} from "@/platform/contracts"

export const platformServices: PlatformServices = {
  target: "web",
  chat: {
    stream(request, { signal, onEvent }) {
      return streamCurioChat(
        __CURIO_CHAT_WORKER_URL__,
        request,
        signal,
        onEvent,
      )
    },
  },
  async openExternalUrl(value) {
    const url = parseExternalHttpUrl(value)
    const opened = window.open(url, "_blank", "noopener,noreferrer")
    if (!opened) {
      throw new Error("The browser blocked Curio from opening this link.")
    }
  },
}
