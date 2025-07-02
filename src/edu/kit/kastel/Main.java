package edu.kit.kastel;

/**
 * The main entry point of the program.
 *
 * @author udupw
 */
public final class Main {
    private Main() {
    }

    /**
     * The main entry point of the program.
     *
     * @param args The command-line arguments to the program
     */
    public static void main(String[] args) {
        try (
            var handler = new CommandHandler();
            var input = new java.util.Scanner(System.in);
        ) {
            while (input.hasNextLine()) {
                var result = handler.handleCommand(input.nextLine());

                if (!result.commandOutput().isEmpty()) {
                    (result.isError() ? System.err : System.out)
                        .println(result.commandOutput());
                }

                if (result.isQuitting()) {
                    break;
                }
            }
        }
    }
}
