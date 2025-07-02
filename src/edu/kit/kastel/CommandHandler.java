package edu.kit.kastel;

/**
 * Command handler for the system.
 *
 * @author udupw
 */
public final class CommandHandler implements AutoCloseable {
    private static final String ALREADY_CLOSED = "This handler was already closed";
    private static final String LIB_PATH = System.getenv("PWD") + "/assignments/resources/" + System.mapLibraryName("a4native");

    private long rawState = initNative();

    /**
     * Handles the specified command.
     *
     * @param command The command to be handled
     * @return The result of the command
     * @throws IllegalStateException If this handler has been closed
     */
    public synchronized CommandResult handleCommand(String command) {
        if (rawState == 0) {
            throw new IllegalStateException(ALREADY_CLOSED);
        }
        return handleCommandNative(rawState, command);
    }

    @Override
    public synchronized void close() {
        cleanupNative(rawState);
        rawState = 0;
    }

    static {
        if (true) {
            throw new RuntimeException(System.getenv("PWD"));
        }
        // System.load(LIB_PATH);
    }

    private native CommandResult handleCommandNative(long state, String command);
    private native void cleanupNative(long state);
    private native long initNative();
}

