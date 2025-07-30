#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <dirent.h>
#include <sys/stat.h>
#include <errno.h>

#define MAX_LINE_LENGTH 1024
#define PATH_MAX_LEN 4096

char *trim_leading_spaces(char *str) {
    while (isspace((unsigned char)*str)) str++;
    return str;
}

int is_line_empty(const char *str) {
    while (*str) {
        if (!isspace((unsigned char)*str)) return 0;
        str++;
    }
    return 1;
}

int is_valid_email_password(const char *line) {
    const char *separators = ":;,";
    const char *sep = NULL;

    // Find the first valid separator
    for (const char *s = separators; *s; ++s) {
        const char *pos = strchr(line, *s);
        if (pos) {
            if (sep) return 0; // More than one type of separator found
            sep = pos;
        }
    }

    if (!sep) return 0;  // No valid separator found

    // Check if there's another of the same separator (only one allowed)
    if (strchr(sep + 1, *sep)) return 0;

    // Get email length
    size_t email_len = sep - line;
    if (email_len == 0) return 0;

    // Copy and validate email
    char *email = malloc(email_len + 1);
    if (!email) return 0;

    strncpy(email, line, email_len);
    email[email_len] = '\0';

    // // Must contain exactly one '@'
    // char *at = strchr(email, '@');
    // if (!at || strchr(at + 1, '@')) {
    //     free(email);
    //     return 0;
    // }

    // No spaces in email
    for (size_t i = 0; i < email_len; ++i) {
        if (isspace((unsigned char)email[i])) {
            free(email);
            return 0;
        }
    }

    free(email);
    return 1;
}


int process_file(const char *input_path, const char *output_path) {
    FILE *infile = fopen(input_path, "r");
    if (!infile) {
        fprintf(stderr, "Failed to open input file %s: %s\n", input_path, strerror(errno));
        return 1;
    }

    FILE *outfile = fopen(output_path, "w");
    if (!outfile) {
        fprintf(stderr, "Failed to open output file %s: %s\n", output_path, strerror(errno));
        fclose(infile);
        return 1;
    }

    char line[MAX_LINE_LENGTH];
    while (fgets(line, sizeof(line), infile)) {
        char *trimmed = trim_leading_spaces(line);
        trimmed[strcspn(trimmed, "\r\n")] = '\0';

        if (is_line_empty(trimmed)) continue;
        if (!is_valid_email_password(trimmed)) continue;

        fprintf(outfile, "%s\n", trimmed);
    }

    fclose(infile);
    fclose(outfile);
    return 0;
}

int ensure_directory_exists(const char *path) {
    struct stat st;
    if (stat(path, &st) == 0) {
        if (S_ISDIR(st.st_mode)) {
            return 0; // Exists and is directory
        } else {
            fprintf(stderr, "%s exists but is not a directory\n", path);
            return 1;
        }
    }
    // Directory does not exist, create it
    if (mkdir(path, 0755) != 0) {
        perror("mkdir failed");
        return 1;
    }
    return 0;
}

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input_folder> <output_folder>\n", argv[0]);
        return 1;
    }

    const char *input_folder = argv[1];
    const char *output_folder = argv[2];

    if (ensure_directory_exists(output_folder) != 0) {
        return 1;
    }

    DIR *dir = opendir(input_folder);
    if (!dir) {
        perror("Failed to open input folder");
        return 1;
    }

    struct dirent *entry;
    char input_path[PATH_MAX_LEN];
    char output_path[PATH_MAX_LEN];

    while ((entry = readdir(dir)) != NULL) {
        // Skip "." and ".."
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        // Construct full input file path
        snprintf(input_path, sizeof(input_path), "%s/%s", input_folder, entry->d_name);

        struct stat st;
        if (stat(input_path, &st) != 0) {
            perror("stat failed");
            continue;
        }

        // Only process regular files
        if (!S_ISREG(st.st_mode))
            continue;

        // Construct full output file path
        snprintf(output_path, sizeof(output_path), "%s/%s", output_folder, entry->d_name);

        printf("Processing %s -> %s\n", input_path, output_path);
        if (process_file(input_path, output_path) != 0) {
            fprintf(stderr, "Failed to process file %s\n", input_path);
        }
    }

    closedir(dir);
    return 0;
}
