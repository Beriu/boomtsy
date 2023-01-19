import {RequestHandler} from 'express';
import JWT from "jsonwebtoken";

const isAuthenticated: RequestHandler = (req, res, next) => {

    const authHeader = req.headers['authorization'];
    const token = authHeader?.split(' ')[1];

    if (token == null) return res.sendStatus(401);

    JWT.verify(token, process.env.TOKEN_SECRET as string, (err: any, user: any) => {

        console.log(user);

        if (err) return res.sendStatus(403);

        next();
    });
};

export default isAuthenticated;

