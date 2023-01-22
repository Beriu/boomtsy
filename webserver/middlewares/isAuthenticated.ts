import {RequestHandler} from 'express';
import JWT from "jsonwebtoken";

const isAuthenticated: RequestHandler = (req, res, next) => {

    const secret = process.env.DISCORD_CLIENT_SECRET as string;
    const authHeader = req.headers['authorization'];
    const token = authHeader?.split(' ')[1];

    if (token == null) return res.sendStatus(401);

    JWT.verify(token, secret, (err: any, user: any) => {

        if (err) return res.sendStatus(403);

        //@ts-ignore
        const { token: discordToken } = JWT.decode(token, secret, { json: true });
        //@ts-ignore
        req.discordToken = discordToken;

        next();
    });
};

export default isAuthenticated;

