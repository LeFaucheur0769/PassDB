#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <dirent.h>
#include <sys/stat.h>
#include <errno.h>

#define MAX_LINE_LENGTH 1024
#define PATH_MAX_LEN 4096

/**
 * @brief Trims leading whitespace characters from a string
 * @details This function advances the pointer to skip over any leading
 * whitespace characters in the input string. The function assumes that
 * the input string is null-terminated.
 * @param str Pointer to the string to be trimmed
 * @return Pointer to the first non-whitespace character in the string
 */
char *trim_leading_spaces(char *str) {
    // Increment the pointer while the current character is a whitespace
    while (isspace((unsigned char)*str)) {
        str++;
    }
    // Return the pointer to the first non-whitespace character
    return str;
}

/**
 * @brief Checks if a line is empty
 * @details A line is considered empty if it only contains whitespace
 * characters.
 * @param str Pointer to the line to check
 * @return 1 if the line is empty, 0 otherwise
 */
int is_line_empty(const char *str) {
    // Iterate over the line until the end is reached
    while (*str) {
        // If a non-whitespace character is encountered, the line is not empty
        if (!isspace((unsigned char)*str)) return 0;
        // Move pointer to the next character
        str++;
    }
    // If the end of the line is reached, the line is empty
    return 1;
}

/**
 * @brief Check if a line is a valid email:password pair
 * @details A valid line is a line that contains a separator (one of :,;,)
 * and where the separator is not mixed with other separators.
 * The email part must not contain any whitespace.
 * @param line The line to check
 * @return 1 if the line is a valid email:password pair, 0 otherwise
 */
int is_valid_email_password(const char *line) {
    const char *separators = ":;,";
    char detected_sep = 0;

    // Scan for first separator and ensure no mixed separators
    for (const char *p = line; *p; ++p) {
        if (strchr(separators, *p)) {
            if (!detected_sep) {
                detected_sep = *p;
            } else if (*p != detected_sep) {
                return 0;  // Mixed separators
            }
        }
    }

    if (!detected_sep) return 0;  // No separator at all

    // Get email part
    const char *sep_pos = strchr(line, detected_sep);
    if (!sep_pos || sep_pos == line) return 0; // No email

    size_t email_len = sep_pos - line;
    char *email = malloc(email_len + 1);
    if (!email) return 0;

    strncpy(email, line, email_len);
    email[email_len] = '\0';

    // Check for whitespace in email
    for (size_t i = 0; i < email_len; ++i) {
        if (isspace((unsigned char)email[i])) {
            free(email);
            return 0;
        }
    }

    free(email);
    return 1;
}


/**
 * @brief Process a single file.
 * @details Process a single file by reading the contents of the file and
 * writing the processed lines to the output file.
 * @param input_path The path to the input file.
 * @param output_path The path to the output file.
 * @return 0 if the file was processed successfully, 1 otherwise.
 */
int process_file(const char *input_path, const char *output_path) {
    // Open the input file
    FILE *infile = fopen(input_path, "r");
    if (!infile) {
        fprintf(stderr, "Failed to open input file %s: %s\n", input_path, strerror(errno));
        return 1;
    }

    // Open the output file
    FILE *outfile = fopen(output_path, "w");
    if (!outfile) {
        fprintf(stderr, "Failed to open output file %s: %s\n", output_path, strerror(errno));
        fclose(infile);
        return 1;
    }

    char line[MAX_LINE_LENGTH];
    // Process each line of the file
    while (fgets(line, sizeof(line), infile)) {
        // Trim leading spaces from the line
        char *trimmed = trim_leading_spaces(line);

        // Remove the newline from the line
        trimmed[strcspn(trimmed, "\r\n")] = '\0';

        // Skip empty lines
        if (is_line_empty(trimmed)) continue;

        // Skip lines that are not valid email:password pairs
        if (!is_valid_email_password(trimmed)) continue;

        // Write the processed line to the output file
        fprintf(outfile, "%s\n", trimmed);
    }

    // Close the input and output files
    fclose(infile);
    fclose(outfile);
    return 0;
}

/**
 * @brief Check if a directory exists.
 * @param path The path to the directory.
 * @return 0 if the directory exists, 1 otherwise.
 */
int ensure_directory_exists(const char *path) {
    struct stat st;
    if (stat(path, &st) == 0) {
        // If the file exists, check if it is a directory
        if (S_ISDIR(st.st_mode)) return 0;
        // If the file exists but is not a directory, return an error
        fprintf(stderr, "%s exists but is not a directory\n", path);
        return 1;
    }
    // If the file does not exist, try to create it
    if (mkdir(path, 0755) != 0) {
        // If the creation failed, return an error
        perror("mkdir failed");
        return 1;
    }
    // If the creation succeeded, return success
    return 0;
}

/**
 * @brief Recursively traverse directories and process files.
 * @details This function traverses the directory tree starting from the
 * base_input path, processes each regular file encountered, and writes
 * the output to the base_output path using a flattened file name.
 * @param base_input The base input directory path.
 * @param base_output The base output directory path.
 * @param relative_path The relative path for recursive traversal.
 */
void traverse_and_process(const char *base_input, const char *base_output, const char *relative_path) {
    char current_path[PATH_MAX_LEN];
    snprintf(current_path, sizeof(current_path), "%s/%s", base_input, relative_path);

    DIR *dir = opendir(current_path);
    if (!dir) {
        perror("opendir failed");
        return;
    }

    struct dirent *entry;
    while ((entry = readdir(dir))) {
        // Skip the current and parent directory entries
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        char rel_path[PATH_MAX_LEN];
        // Construct the relative path
        if (strlen(relative_path) > 0)
            snprintf(rel_path, sizeof(rel_path), "%s/%s", relative_path, entry->d_name);
        else
            snprintf(rel_path, sizeof(rel_path), "%s", entry->d_name);

        char full_input_path[PATH_MAX_LEN];
        snprintf(full_input_path, sizeof(full_input_path), "%s/%s", base_input, rel_path);

        struct stat st;
        if (stat(full_input_path, &st) != 0) {
            perror("stat failed");
            continue;
        }

        if (S_ISDIR(st.st_mode)) {
            // Recursively process subdirectories
            traverse_and_process(base_input, base_output, rel_path);
        } else if (S_ISREG(st.st_mode)) {
            // Flatten the output file name (replace slashes with double underscores)
            char flat_name[PATH_MAX_LEN];
            snprintf(flat_name, sizeof(flat_name), "%s", rel_path);
            for (char *p = flat_name; *p; ++p)
                if (*p == '/') *p = '_';

            char output_file_path[PATH_MAX_LEN];
            snprintf(output_file_path, sizeof(output_file_path), "%s/%s", base_output, flat_name);

            printf("Processing %s -> %s\n", full_input_path, output_file_path);
            // Process the file and handle any errors
            if (process_file(full_input_path, output_file_path) != 0) {
                fprintf(stderr, "Failed to process file %s\n", full_input_path);
            }
        }
    }

    closedir(dir);
}

/**
 * @brief Main entry point for the program.
 *
 * This program takes two command line arguments, an input folder and an output
 * folder. It traverses the input folder and processes all regular files found
 * in it, and writes the processed files to the output folder.
 *
 * @param argc The number of command line arguments.
 * @param argv The command line arguments.
 * @return 0 on success, 1 on failure.
 */
int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input_folder> <output_folder>\n", argv[0]);
        return 1;
    }

    const char *input_folder = argv[1];
    const char *output_folder = argv[2];

    if (ensure_directory_exists(output_folder) != 0)
        return 1;

    /**
     * Start the traversal of the input folder and its subfolders. The empty
     * string as the third argument means that the traversal starts from the
     * root of the input folder.
     */
    traverse_and_process(input_folder, output_folder, "");

    return 0;
}
