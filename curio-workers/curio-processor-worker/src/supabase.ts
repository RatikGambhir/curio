import { createClient } from "@supabase/supabase-js";



export const genSupabaseClient = (env: Env) => {
	const supabaseUrl = env.SUPABASE_URL;
	const supabaseAnonKey  = env.SUPABASE_KEY


	return createClient(supabaseUrl, supabaseAnonKey, {
		db: { schema: "messaging"},
		auth: {
			persistSession: true,
			autoRefreshToken: true,
			detectSessionInUrl: true,
		},
	});

}
