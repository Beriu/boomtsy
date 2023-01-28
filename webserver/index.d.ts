// to make the file a module and avoid the TypeScript error
export {}

declare global {
    namespace Express {
        export interface Request {
            context?: {
                discordToken: string,
                user: {
                    id: string,
                    avatar: string,
                    username: string,
                }
            }
        }
    }
}