import os
import re

directory = 'mobile/composeApp/src/commonTest/kotlin'
for root, _, files in os.walk(directory):
    for file in files:
        if file.endswith('.kt'):
            path = os.path.join(root, file)
            with open(path, 'r') as f:
                content = f.read()

            if '"updated_at"' in content:
                # We want to replace `"updated_at": "..."` with `"updated_at": "...", "progress_percent": null, "max_progress_percent": null, "last_read_at": null, "finished_at": null`
                # Only if they are not already there.
                if '"progress_percent"' not in content:
                    content = re.sub(r'("updated_at": ".*?Z")', r'\1,\n"progress_percent": null,\n"max_progress_percent": null,\n"last_read_at": null,\n"finished_at": null', content)
                    with open(path, 'w') as f:
                        f.write(content)
