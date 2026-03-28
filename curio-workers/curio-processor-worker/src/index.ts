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

const queueBody: QueueBody = {
	userId: "266ee938-12db-47d1-9ffd-6d53d0b25808",
	userMessageId: "cd7eb020-3e46-467d-a4e5-5a3dab7db25b",
	threadId: "18b2c081-60d6-4357-82b6-e786453fbb0d",
	assistantMessageId: "46193a0e-9dec-433b-8445-79968ac175fe",
	assetPath: ["chat/body", "chat/body2"],

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


const queryEmbedding = async (content: string) => {
	return await gemini.models.embedContent({
		model: "gemini-embedding-001",
		contents: content
	}).then((result) => Success(result as EmbedContentResponse))
		.catch(err => {
			console.log("ERROR: Embedding API: ", err)
			return Failure(err)
		})
}

const embeddContent = async (content: Success<MessageResult[]>) => {
	return await Promise.all(
		content.value.map((val) => {
			return queryEmbedding(val.content)
		})
	)

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

const queryContent = async (body: QueueBody, supabase: DBClient): Promise<Result<MessageResult[]>> => {
	try {
		const {data, error} = await supabase.schema("messaging").from("messages")
			.select('id, role, content')
			.in('id', [body.userMessageId, body.assistantMessageId])
		if (error) {
			return Failure(error)
		}
		return Success(data as MessageResult[])
	} catch (error) {
		return Failure(error as Error)
	}

}



const genAssetEmbeddings = async (assets: Result<Blob, Error>[]) => {
	return await Promise.all(
		assets.map(async (asset) => {
			if (!asset.ok) {
				return asset
			}
			return await queryEmbedding(await encode(asset.value))
		})
	)
}


const queryAsset = async (path: string, supabase: DBClient): Promise<Result<Blob>> => {
	try {
		const {data, error} = await supabase.storage.from("user_assets").download(path)
		if (error) {
			return Failure(error)
		}
		return Success(data as Blob)
	} catch (error) {
		return Failure(error as Error)
	}
}
const queryAssets = async (paths: string[], supabase: DBClient): Promise<Result<Blob>[]> => {
	return await Promise.all(
		paths.map(async (path) => {
			return queryAsset(path, supabase)

		})
	).then(data => data);
}


export default {
	async queue(batch, env, ctx): Promise<void> {
		const geminiClient = new GoogleGenAI({apiKey: env.GEMINI_API_KEY})
		const supabaseClient = genSupabaseClient(env)
		for (const message of batch.messages) {
			const assetResult = await queryAssets(queueBody.assetPath, supabaseClient).then(assetResults => genAssetEmbeddings(assetResults))

			const contentResult = await queryContent(queueBody, supabaseClient).then((result) => {
				if (result.ok) {
					return embeddContent(result)
				}

			})

			contentResult?.forEach((result: { ok: any; value: { metadata: any; embeddings: any; }; error: any; }) => {
				if (result.ok) {
					console.log("Content Embeddings Metadata: ", result.value.metadata)

					console.log("Content Embeddings: ", result.value.embeddings)

				} else {
					console.log("ERROR: Asset Embeddings: ", result.error)

				}

			})

			assetResult.forEach((assetResult: { ok: any; value: { metadata: any; embeddings: any; }; error: any; } ) => {
				if (assetResult.ok) {
					console.log("Asset Embeddings Metadata: ", assetResult.value.metadata)

					console.log("Asset Embeddings: ", assetResult.value.embeddings)
				} else {
					console.log("ERROR: Asset Embeddings: ", assetResult.error)

				}

			})

		}
	},
} satisfies ExportedHandler<Env>;
