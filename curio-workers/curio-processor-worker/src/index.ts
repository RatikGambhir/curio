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
	SUPABASE_URL: string,
	SUPABASE_KEY: string,
}

type DBClient = SupabaseClient<any, "public", "messaging", any, any>

interface MessageBody {
	userId: string
	userMessageId: string
	threadId: string
	assistantMessageId: string,
	assetBucketId: string

}

import {EmbedContentResponse} from "@google/genai";
import {genSupabaseClient} from "./supabase";
import {SupabaseClient} from "@supabase/supabase-js";

interface QueueBody {
	userId: string
	userMessageId: string
	threadId: string
	assistantMessageId: string,
	assetPath: string[]
}

type Success<T> = {ok: true, value: T}
type Failure<E = Error> = {ok: false, error: E}

type Result<T, E = Error> = Success<T> | Failure<E>

type MessageResult = {
	id: string,
	role: string,
	content: string
}
const Success = <T>(val: T) => {
	return {
		ok: true,
		value: val
	} as Success<T>
}

const Failure = (error: Error) => {
	return {
		ok: false,
		error: error
	} as Failure
}

const queryEmbedding = async (content: string, geminiClient: GoogleGenAI) => {
	return await geminiClient.models.embedContent({
		model: "gemini-embedding-001",
		contents: content
	}).then((result) => Success(result as EmbedContentResponse))
		.catch(err => {
			console.log("ERROR: Embedding API: ", err)
			return Failure(err)
		})
}


const genContentEmbeddings = async (content: Success<MessageResult[]>, geminiClient: GoogleGenAI): Promise<Result<EmbedContentResponse[], Error>> => {
	const result = await Promise.all(
		content.value.map((val) => {
			return queryEmbedding(val.content, geminiClient)
		})
	)
	const failed = result.find((result) => !result.ok)
	if (failed) {
		return Failure(failed.error)
	}
	const succeeded = result.map((embedding) => {
		if (embedding.ok) {
			return embedding.value
		}
	}).filter(x => x !== undefined)

	return Success(succeeded)
}


const encode = async (blob: Blob) => {
	const bytes = new Uint8Array(await blob.bytes())
	let binary = ""
	const chunkSize = 0x8000;
	for (let i = 0; i < bytes.length;  i += chunkSize) {
		binary += String.fromCharCode(...bytes.subarray(i,  i + chunkSize))
	}
	return btoa(binary)
}

const queryContent = async (body: QueueBody, dbClient: DBClient): Promise<Result<MessageResult[]>> => {
	try {
		const {data, error} = await dbClient.schema("messaging").from("messages")
			.select('id, role, content')
			.in('id', [body.userMessageId, body.assistantMessageId])
		if (error) {
			console.log("FAILURE: ", body.userMessageId)

			return Failure(error)
		}
		return Success(data as MessageResult[])
	} catch (error) {
		console.log("FAILURE: ", body.userMessageId)

		return Failure(error as Error)
	}

}

const genAssetEmbeddings = async (
	assets: Success<Blob[]>,
	geminiClient: GoogleGenAI
): Promise<Result<EmbedContentResponse[], Error>> => {
	const results = await Promise.all(
		assets.value.map(async (blob) => {
			const encoded = await encode(blob)
			return queryEmbedding(encoded, geminiClient)
		})
	)
	const failed = results.find((result) => !result.ok)
	if (failed) {
		return Failure(failed.error)
	}
	const succeeded = results.map((embedding) => {
		if (embedding.ok) {
			return embedding.value
		}
	}).filter(x => x !== undefined)
	return Success(succeeded)
}


const queryAsset = async (path: string, dbClient: DBClient): Promise<Result<Blob>> => {
	try {
		const {data, error} = await dbClient.storage.from("user_assets").download(path)
		if (error) {
			console.log("FAILURE: ", path)
			return Failure(error)
		}
		return Success(data as Blob)
	} catch (error) {
		console.log("FAILURE: ", path)

		return Failure(error as Error)
	}
}


const queryAssets = async (paths: string[], dbClient: DBClient): Promise<Result<Blob[]>> => {
	return await Promise.all(
		paths.map(async (path) => {
			return queryAsset(path, dbClient)
		})
	).then(data => {
		const blob: Blob[] = []
		data.forEach((result) => {
			if (!result.ok) {
				return Failure(result.error)
			}
			blob.push(result.value)
		})
		return Success(blob)
	});
}

export default {
	async queue(batch: MessageBatch<QueueBody>, env: Env, ctx: ExecutionContext): Promise<void> {
		const geminiClient = new GoogleGenAI({apiKey: env.GEMINI_API_KEY})
		const supabaseClient = genSupabaseClient(env)
		for (const message of batch.messages) {
			const assetResult = await queryAssets(message.body.assetPath, supabaseClient)
			const content = await queryContent(message.body, supabaseClient)
			if (content.ok && assetResult.ok) {
				console.log("RESULT: ", content.value)
				assetResult.value.forEach((result) => {
					console.log("RESULT: ", result)
				})
				content.value.forEach((message) => {
					console.log("RESULT: ", message.content)
				})
				const embeddedContent = await genContentEmbeddings(content, geminiClient)
				const embeddedAssets = await genAssetEmbeddings(assetResult, geminiClient)
				if (embeddedAssets.ok && embeddedContent.ok) {
					// Save here
				} else {
					console.error("QUEUE FAILED")
					message.retry()
				}


			} else {
				console.error("QUEUE FAILED")
				message.retry()
			}

		}
	},
} satisfies ExportedHandler<Env, QueueBody>;
