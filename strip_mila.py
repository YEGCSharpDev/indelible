import re

with open('mobile/composeApp/src/commonTest/kotlin/app/indelible/core/network/ApiClient.kt', 'r') as f:
    lines = f.readlines()

new_lines = []
skip = False
for line in lines:
    if 'Mila' in line or 'mila' in line:
        if 'fun ' in line and '=' not in line:
            skip = True
        continue
    if skip:
        if '=' in line or '{' in line:
            skip = False
        continue
    new_lines.append(line)

with open('mobile/composeApp/src/commonTest/kotlin/app/indelible/core/network/ApiClient.kt', 'w') as f:
    f.writelines(new_lines)
