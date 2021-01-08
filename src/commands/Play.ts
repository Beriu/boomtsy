import {slashCommand, slashCommandOptionType} from "../types";

const command: slashCommand = {
    name: 'play',
    description: 'Stream the audio of a youtube video.',
    options: [
        {
            name: 'input',
            description: 'Youtube video or keywords like song title and/or author.',
            required: true,
            type: slashCommandOptionType.STRING
        }
    ]
};
export default command;