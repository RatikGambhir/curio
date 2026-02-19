import type { RegisterData } from "@/types/LoginRegisterTypes";

const registerUser = async (registerData: RegisterData) => {
    void registerData;
    // const response = await fetch(`${import.meta.env.VITE_API_URL}/auth/register`, {
    //     method: "POST",
    //     body: JSON.stringify(registerData),
    // });
    // return response.json();
};

export { registerUser };
