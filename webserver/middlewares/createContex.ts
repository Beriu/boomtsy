import {RequestHandler} from 'express';

const createContex = (): RequestHandler => {
    return (req, res, next) => {
    
        const ctx = {
            user: {
                id: "",
                avatar: "",
                username: "",
            }, 
            discordToken: ""
        };
    
        if(!req.context) req.context = ctx;
    
        next();
    }
};

export default createContex;