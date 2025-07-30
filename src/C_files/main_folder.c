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

    for (const char *s = separators; *s; ++s) {
        const char *pos = strchr(line, *s);
        if (pos) {
            if (sep) return 0;
            sep = pos;
        }
    }

    if (!sep) return 0;
    if (strchr(sep + 1, *sep)) return 0;

    size_t email_len = sep - line;
    if (email_len == 0) return 0;

    char *email = malloc(email_len + 1);
    if (!email) return 0;

    strncpy(email, line, email_len);
    email[email_len] = '\0';

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
        if (S_ISDIR(st.st_mode)) return 0;
        fprintf(stderr, "%s exists but is not a directory\n", path);
        return 1;
    }
    if (mkdir(path, 0755) != 0) {
        perror("mkdir failed");
        return 1;
    }
    return 0;
}

// Recursively traverse and process
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
        if (strcmp(entry->d_name, ".") == 0 || strcmp(entry->d_name, "..") == 0)
            continue;

        char rel_path[PATH_MAX_LEN];
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
            if (process_file(full_input_path, output_file_path) != 0) {
                fprintf(stderr, "Failed to process file %s\n", full_input_path);
            }
        }
    }

    closedir(dir);
}

int main(int argc, char *argv[]) {
    if (argc != 3) {
        fprintf(stderr, "Usage: %s <input_folder> <output_folder>\n", argv[0]);
        return 1;
    }

    const char *input_folder = argv[1];
    const char *output_folder = argv[2];

    if (ensure_directory_exists(output_folder) != 0)
        return 1;

    traverse_and_process(input_folder, output_folder, "");

    return 0;
}
