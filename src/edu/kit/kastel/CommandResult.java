package edu.kit.kastel;

/**
 * Record that describes the output of a command.
 *
 * @param commandOutput The output message of the command
 * @param isError Whether the command errored
 * @param isQuitting Whether the application should quit after handling the command
 *
 * @author udupw
 */
public record CommandResult(String commandOutput, boolean isError, boolean isQuitting) {
}
