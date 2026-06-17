// {{project_name}} - Raylib Project
#include "raylib.h"

int main() {
    const int screenWidth = 800;
    const int screenHeight = 600;

    InitWindow(screenWidth, screenHeight, "{{project_name}}");
    SetTargetFPS(60);

    while (!WindowShouldClose()) {
        // Update

        // Draw
        BeginDrawing();
            ClearBackground(RAYWHITE);
            DrawText("Hello from {{project_name}}!", 190, 200, 20, LIGHTGRAY);

            DrawCircle(screenWidth / 2, screenHeight / 2, 50, MAROON);
        EndDrawing();
    }

    CloseWindow();
    return 0;
}
