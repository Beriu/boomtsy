import { request } from "undici";

export const authenticate = async (code: string, redirectUri: string) => {

    console.log(`Authentication Request code: ${code}`);

    const body = new URLSearchParams({
        client_id: process.env.DISCORD_APP_ID as string,
        client_secret: process.env.DISCORD_CLIENT_SECRET as string,
        code,
        grant_type: 'authorization_code',
        redirect_uri: redirectUri,
        scope: 'identify connections',
    }).toString();
    
    try {
        const rawRequest = await request('https://discord.com/api/oauth2/token', {
            method: 'POST',
            body,
            headers: { 'Content-Type': 'application/x-www-form-urlencoded' },
        });
        const response = await rawRequest.body.json();
        
        if(response.error) throw new Error(response.error.error_description ?? response.error);
        
        console.log(`Authentication Request successfull: `, response);

        return response;

    }catch(error: any) {
        throw new Error(error.message);
    }
};

export const fetchUser = async (token: string) => {

    console.log(`Fetching user data, token: ${token}`);

    const raw = await request('https://discord.com/api/users/@me', {
        headers: { 
            authorization: `Bearer ${token}`
        },
    });
    const user = await raw.body.json();

    console.log(`User data retrieved:`, user);

    return user;
};

// export const getUserConnections = async (token: string) => {

//     console.log(`Fetching user connections, token: ${token}`);

//     const raw = await request('https://discord.com/api/users/@me/connections', {
//         headers: { 
//             authorization: `Bearer ${token}`
//         },
//     });

//     const connections = await raw.body.json();

//     console.log(`Connections retrieved:`, connections);

//     return connections;
// };

