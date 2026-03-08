

const workerURLString = import.meta.env.CHAT_WORKER_URL
const workerURL = new URL(workerURLString ?? "https://api.gettingcurio.com/chat")


export const streamChat = async (question: string, handleChunk: (chunk: string, done: boolean) => void) => {
    const response = await fetch(workerURL, {
        body: JSON.stringify({question}),
        method: "POST",
        headers: { "Content-Type": "application/json" },
    })

    const reader = response?.body?.getReader()
    const decoder = new TextDecoder()

    while (reader) {
        const {value, done } = await reader.read()

        if (value) {
            handleChunk(decoder.decode(value), done)
        }
    }
}

