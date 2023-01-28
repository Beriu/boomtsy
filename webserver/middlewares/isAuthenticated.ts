import {RequestHandler} from 'express';
import JWT from "jsonwebtoken";

const isAuthenticated: RequestHandler = (req, res, next) => {

    const secret = process.env.DISCORD_CLIENT_SECRET as string;
    const authHeader = req.headers['authorization'];
    const token = authHeader?.split(' ')[1];

    if (token == null) return res.sendStatus(401);

    JWT.verify(token, secret, (err: any) => {

        if (err) return res.sendStatus(403);

        //@ts-ignore
        const { token: discordToken, user } = JWT.decode(token, secret, { json: true });
        
        req.context!.discordToken = discordToken;
        req.context!.user = user;

        next();
    });
};

export default isAuthenticated;

