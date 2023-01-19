import { request } from "undici";

export const authenticate = async (code: string) => {

    const body = new URLSearchParams({
        client_id: process.env.DISCORD_APP_ID as string,
        client_secret: process.env.DISCORD_CLIENT_SECRET as string,
        code,
        grant_type: 'authorization_code',
        redirect_uri: `http://localhost:8080`,
        scope: 'identify',
    }).toString();

    try {
        const rawRequest = await request('https://discord.com/api/oauth2/token', {
            method: 'POST',
            body,
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        });
        const response = await rawRequest.body.json();
        
        if(response.error) throw new Error(response.error.error_description ?? response.error);
        
        return response;

    }catch(error: any) {
        throw new Error(error.message);
    }
};