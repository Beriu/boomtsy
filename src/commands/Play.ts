import {slashCommand, slashCommandOptionType} from "../types";

const command: slashCommand = {
    name: 'Play',
    description: 'Stream the audio of a youtube video.',
    options: [
        {
            name: 'url',
            description: 'Needs to be a public youtube video url.',
            required: true,
            type: slashCommandOptionType.STRING
        }
    ]
};
export default command;