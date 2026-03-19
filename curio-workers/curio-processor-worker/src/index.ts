/**
 * Welcome to Cloudflare Workers! This is your first worker.
 *
 * - Run `npm run dev` in your terminal to start a development server
 * - Open a browser tab at http://localhost:8787/ to see your worker in action
 * - Run `npm run deploy` to publish your worker
 *
 * Bind resources to your worker in `wrangler.jsonc`. After adding bindings, a type definition for the
 * `Env` object can be regenerated with `is`.
 *
 * Learn more at https://developers.cloudflare.com/workers/
 */

import {GoogleGenAI} from "@google/genai";

interface Env {
	GEMINI_API_KEY: string
}

interface User {
	id: string
}


interface MessageBody {
	userId: string
	userMessageId: string
	threadId: string
	assistantMessageId: string,
	assetBucketId: string

}

const queryPromptData = async (messageId: string, threadId: string,  userId: string) => {

}

const queryAssetData = async (path: string) => {

}

const genEmbeddings = async (data: any) => {

}

const genPromptEmbeddings = async (promptData: MessageBody) => {

}




export default {
	async queue(batch, env, ctx): Promise<void> {
		const geminiClient = new GoogleGenAI({apiKey: env.GEMINI_API_KEY})
		for (const message of batch.messages) {
			console.log("consumed from our queue:");

			const messageBody = message.body as MessageBody
			console.log("paresed question: ", messageBody.userId ?? "NOT PARESE");

			const embeddings = await geminiClient.models.embedContent({
				model: "gemini-embedding-001",
				contents: messageBody.assistantMessageId
			})

			console.log("EMBEDDINGS: ", embeddings)

			//TODO: SAVE TO SUPABASE HERE

		}
	},
} satisfies ExportedHandler<Env>;
