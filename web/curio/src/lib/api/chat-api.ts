

const workerURLString = import.meta.env.CHAT_WORKER_URL
const workerURL = new URL(workerURLString ?? "https://api.gettingcurio.com/chat")


export const streamChat = async (question: string, acc: (validators: string) => void) => {
    const response = await fetch(workerURL, {
        body: JSON.stringify({prompt: question}),
        method: "POST",
        headers: { "Content-Type": "application/json" },
    })
    if (!response.ok) {
        const text = await response.text();
        throw new Error(`HTTP ${response.status}: ${text}`);
    }

    if (!response.body) {
        throw new Error("No response body");
    }

    const reader = response.body.getReader()
    const decoder = new TextDecoder()

    while (true) {
            const {value, done } = await reader.read()
            if (done || !value) {
                break;
            }
            if (value) {
                const chunk = decoder.decode(value, {stream: true})
                acc(chunk)
            }
        }

}

