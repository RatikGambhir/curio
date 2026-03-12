import { createClient } from "@supabase/supabase-js";



export const genSupabaseClient = (env: Env) => {
	const supabaseUrl = env.SUPABASE_URL;
	const supabaseAnonKey  = env.SUPABASE_ANON_KEY


	return createClient(supabaseUrl, supabaseAnonKey, {
		auth: {
			persistSession: true,
			autoRefreshToken: true,
			detectSessionInUrl: true,
		},
	});

}
