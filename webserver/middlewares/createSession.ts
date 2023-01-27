import {RequestHandler} from 'express';

const createSession = (): RequestHandler => {
    return (req, res, next) => {
    
        const session = {
            user: {}, 
            discordToken: undefined
        };
    
        if(!req.session) req.session = session;
    
        next();
    }
};

export default createSession;