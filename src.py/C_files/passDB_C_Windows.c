#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <sys/stat.h>
#include <windows.h>

#define MAX_LINE_LEN 4096
#define MAX_WORDS 512

// Utility: lowercase first 3 chars of string
/**
 * @brief Get the first 3 char of a string, lowercased
 * @param line The string to get the prefix from
 * @param prefix_out A buffer to store the prefix in
 */
void get_prefix(const char *line, char *prefix_out) {
    int i, j = 0;
    // Iterate over the first three characters of the input string
    for (i = 0; i < 3 && line[i] != '\0'; i++) {
        // Check if the character is alphanumeric
        if (isalnum(line[i])) {
            // Convert to lowercase and store in the output buffer
            prefix_out[j++] = tolower(line[i]);
        }
    }
    // Null-terminate the output prefix
    prefix_out[j] = '\0';
}

// Utility: trim newline and trailing spaces
/**
 * @brief Trim newline and trailing spaces from a string
 * @param line The string to trim
 */
void trim_line(char *line) {
    size_t len = strlen(line);
    // Iterate backwards over the string until we find a non-space
    while (len && (line[len - 1] == '\n' || line[len - 1] == ' ')) {
        // Replace the last character with a null-terminator
        line[--len] = '\0';
    }
}

/**
 * @brief Compare function for qsort
 * 
 * This function compares two strings pointed to by the void pointers
 * and returns an integer indicating their lexicographical order.
 * 
 * @param a Pointer to the first string
 * @param b Pointer to the second string
 * @return Negative value if the first string is less than the second,
 *         zero if they are equal, positive if the first is greater.
 */
int cmpstr(const void *a, const void *b) {
    // Cast and dereference the void pointers to get the actual strings
    return strcmp(*(const char **)a, *(const char **)b);
}

// Process a single file
/**
 * @brief Process a single file
 * 
 * This function reads a single file line by line, sorts the words in each line,
 * and writes the sorted lines to a file with a name derived from the first three
 * characters of the sorted line.
 * 
 * @param filepath The path to the input file
 * @param outdir The directory to write the output files to
 */
void process_file(const char *filepath, const char *outdir) {
    FILE *fp = fopen(filepath, "r");
    if (!fp) {
        perror(filepath);
        return;
    }

    char line[MAX_LINE_LEN];

    while (fgets(line, sizeof(line), fp)) {
        // Trim newline and trailing spaces
        trim_line(line);
        if (strlen(line) == 0) continue;

        // Split into words
        char *words[MAX_WORDS];
        int count = 0;
        char *token = strtok(line, " \t");
        while (token && count < MAX_WORDS) {
            words[count++] = _strdup(token);  // Windows version of strdup
            token = strtok(NULL, " \t");
        }

        if (count == 0) continue;

        // Sort words
        qsort(words, count, sizeof(char *), cmpstr);

        // Rebuild sorted line
        char sorted_line[MAX_LINE_LEN] = "";
        for (int i = 0; i < count; i++) {
            strcat(sorted_line, words[i]);
            if (i < count - 1) strcat(sorted_line, " ");
            free(words[i]);
        }

        // Get prefix
        char prefix[4];
        get_prefix(sorted_line, prefix);
        if (strlen(prefix) < 1) continue;

        // Build output path
        char outpath[MAX_LINE_LEN];
        snprintf(outpath, sizeof(outpath), "%s\\%s.txt", outdir, prefix);
        FILE *outfp = fopen(outpath, "a");
        if (outfp) {
            fprintf(outfp, "%s\n", sorted_line);
            fclose(outfp);
        }
    }

    fclose(fp);
}

// Make output directory if it doesn't exist
/**
 * @brief Ensure that the given path exists as a directory
 * @param path The path to the directory
 */
void ensure_output_dir(const char *path) {
    DWORD ftyp = GetFileAttributesA(path);
    if (ftyp == INVALID_FILE_ATTRIBUTES || !(ftyp & FILE_ATTRIBUTE_DIRECTORY)) {
        // Create the directory
        CreateDirectoryA(path, NULL);
    }
}

// Main
int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input_file_or_dir> <output_dir>\n", argv[0]);
        return 1;
    }

    const char *input_path = argv[1];
    const char *output_dir = argv[2];

    ensure_output_dir(output_dir);

    struct stat path_stat;
    if (stat(input_path, &path_stat) != 0) {
        perror("stat");
        return 1;
    }

    if (path_stat.st_mode & S_IFREG) {
        printf("📄 Processing single file: %s\n", input_path);
        process_file(input_path, output_dir);
    } else if (path_stat.st_mode & S_IFDIR) {
        char search_path[MAX_LINE_LEN];
        snprintf(search_path, sizeof(search_path), "%s\\*.txt", input_path);

        WIN32_FIND_DATAA fd;
        HANDLE hFind = FindFirstFileA(search_path, &fd);

        if (hFind == INVALID_HANDLE_VALUE) {
            fprintf(stderr, "No .txt files found in directory: %s\n", input_path);
            return 1;
        }

        do {
            if (!(fd.dwFileAttributes & FILE_ATTRIBUTE_DIRECTORY)) {
                char filepath[MAX_LINE_LEN];
                snprintf(filepath, sizeof(filepath), "%s\\%s", input_path, fd.cFileName);
                printf("📄 Processing %s\n", fd.cFileName);
                process_file(filepath, output_dir);
            }
        } while (FindNextFileA(hFind, &fd));
        FindClose(hFind);
    } else {
        fprintf(stderr, "✘ %s is not a valid file or directory\n", input_path);
        return 1;
    }

    return 0;
}

