package edu.kit.kastel;

/**
 * Command handler for the system.
 *
 * @author udupw
 */
public final class CommandHandler implements AutoCloseable {
    private static final String ALREADY_CLOSED = "This handler was already closed";
    private static final String LIB_PATH = System.getenv("PWD") + "/assignment/resources/";
    private static final String LIB_FILE = "a4native";
    private static final long INVALID_POINTER = 0;
    private static final long LOADER_WORKAROUND_LIB_COUNT = 200;

    private long rawState = initNative();

    /**
     * Handles the specified command.
     *
     * @param command The command to be handled
     * @return The result of the command
     * @throws IllegalStateException If this handler has been closed
     */
    public synchronized CommandResult handleCommand(String command) {
        if (rawState == INVALID_POINTER) {
            throw new IllegalStateException(ALREADY_CLOSED);
        }
        return handleCommandNative(rawState, command);
    }

    @Override
    public synchronized void close() {
        cleanupNative(rawState);
        rawState = INVALID_POINTER;
    }

    static {
        System.load(LIB_PATH + System.mapLibraryName(LIB_FILE + System.nanoTime() % LOADER_WORKAROUND_LIB_COUNT));
    }

    private static native CommandResult handleCommandNative(long state, String command);
    private static native void cleanupNative(long state);
    private static native long initNative();
}

