#include <windows.h>
#include <dwmapi.h>
#include <dcomp.h>
#include <d3d11.h>
#include <dxgi1_2.h>
#include <d2d1_3.h>
#include <vector>
#include <unordered_map>
#include <cmath>

#pragma comment(lib, "dcomp.lib")
#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "dxgi.lib")
#pragma comment(lib, "d2d1.lib")

struct Color {
    float r, g, b, a;
};

struct BorderData {
    IDCompositionVisual* visual = nullptr;
    HWND targetHwnd = nullptr;
    int borderSize = 2;
    float rounding = 10.0f;
    float angle = 0.0f;
    std::vector<Color> colors;
};

struct CompositorState {
    ID3D11Device* d3dDevice = nullptr;
    IDXGIDevice* dxgiDevice = nullptr;
    ID2D1Factory3* d2dFactory = nullptr;
    ID2D1Device2* d2dDevice = nullptr;
    ID2D1DeviceContext2* d2dContext = nullptr;
    IDCompositionDevice* dcompDevice = nullptr;
    IDCompositionTarget* dcompTarget = nullptr;
    IDCompositionVisual* rootVisual = nullptr;
    std::unordered_map<HWND, BorderData> activeBorders;
};

static CompositorState g_State;

extern "C" {
    __declspec(dllexport) bool InitCompositor(HWND overlayHwnd) {
        D3D_FEATURE_LEVEL featureLevel;
        HRESULT hr = D3D11CreateDevice(nullptr, D3D_DRIVER_TYPE_HARDWARE, nullptr, D3D11_CREATE_DEVICE_BGRA_SUPPORT, 
                                       nullptr, 0, D3D11_SDK_VERSION, &g_State.d3dDevice, &featureLevel, nullptr);
        if (FAILED(hr)) return false;

        hr = g_State.d3dDevice->QueryInterface(__uuidof(IDXGIDevice), (void**)&g_State.dxgiDevice);
        if (FAILED(hr)) return false;

        D2D1_FACTORY_OPTIONS options = {};
        hr = D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, __uuidof(ID2D1Factory3), &options, (void**)&g_State.d2dFactory);
        if (FAILED(hr)) return false;

        hr = g_State.d2dFactory->CreateDevice(g_State.dxgiDevice, &g_State.d2dDevice);
        if (FAILED(hr)) return false;

        hr = g_State.d2dDevice->CreateDeviceContext(D2D1_DEVICE_CONTEXT_OPTIONS_NONE, &g_State.d2dContext);
        if (FAILED(hr)) return false;

        hr = DCompositionCreateDevice(g_State.dxgiDevice, __uuidof(IDCompositionDevice), (void**)&g_State.dcompDevice);
        if (FAILED(hr)) return false;

        hr = g_State.dcompDevice->CreateTargetForHwnd(overlayHwnd, true, &g_State.dcompTarget);
        if (FAILED(hr)) return false;

        hr = g_State.dcompDevice->CreateVisual(&g_State.rootVisual);
        if (FAILED(hr)) return false;

        g_State.dcompTarget->SetRoot(g_State.rootVisual);
        g_State.dcompDevice->Commit();

        return true;
    }

    __declspec(dllexport) void CreateBorder(HWND targetHwnd, int borderSize, float rounding) {
        if (g_State.activeBorders.count(targetHwnd)) return;

        BorderData data = {};
        data.targetHwnd = targetHwnd;
        data.borderSize = borderSize;
        data.rounding = rounding;
        data.colors = {{0.25f, 0.41f, 0.88f, 1.0f}, {0.0f, 0.0f, 0.0f, 1.0f}}; // default gradient

        g_State.dcompDevice->CreateVisual(&data.visual);
        g_State.rootVisual->AddVisual(data.visual, TRUE, nullptr);
        g_State.activeBorders[targetHwnd] = data;
    }

    __declspec(dllexport) void SetBorderColors(HWND targetHwnd, Color* colors, int count) {
        auto it = g_State.activeBorders.find(targetHwnd);
        if (it == g_State.activeBorders.end()) return;
        
        it->second.colors.assign(colors, colors + count);
    }

    __declspec(dllexport) void SetBorderAngle(HWND targetHwnd, float angle) {
        auto it = g_State.activeBorders.find(targetHwnd);
        if (it == g_State.activeBorders.end()) return;
        
        it->second.angle = angle;
    }

    __declspec(dllexport) void UpdateBorderPosition(HWND targetHwnd, int x, int y, int width, int height) {
        auto it = g_State.activeBorders.find(targetHwnd);
        if (it == g_State.activeBorders.end()) return;

        BorderData& data = it->second;
        data.visual->SetOffsetX((float)(x - data.borderSize));
        data.visual->SetOffsetY((float)(y - data.borderSize));

        IDCompositionSurface* surface = nullptr;
        int surfaceWidth = width + (data.borderSize * 2);
        int surfaceHeight = height + (data.borderSize * 2);

        HRESULT hr = g_State.dcompDevice->CreateSurface(surfaceWidth, surfaceHeight, 
                                                        DXGI_FORMAT_B8G8R8A8_UNORM, DXGI_ALPHA_MODE_PREMULTIPLIED, &surface);
        
        if (SUCCEEDED(hr)) {
            POINT offset;
            ID2D1DeviceContext* dc = nullptr;
            hr = surface->BeginDraw(nullptr, __uuidof(ID2D1DeviceContext), (void**)&dc, &offset);
            
            if (SUCCEEDED(hr)) {
                dc->Clear(D2D1::ColorF(0, 0, 0, 0));
                
                // Create Gradient Stops
                std::vector<D2D1_GRADIENT_STOP> stops;
                for (size_t i = 0; i < data.colors.size(); ++i) {
                    stops.push_back({ (float)i / (data.colors.size() - 1), 
                                      D2D1::ColorF(data.colors[i].r, data.colors[i].g, data.colors[i].b, data.colors[i].a) });
                }

                ID2D1GradientStopCollection* pStopCollection = nullptr;
                dc->CreateGradientStopCollection(stops.data(), (UINT32)stops.size(), &pStopCollection);

                if (pStopCollection) {
                    // Calculate linear gradient points based on angle
                    float rad = data.angle * (3.14159f / 180.0f);
                    float cx = surfaceWidth / 2.0f;
                    float cy = surfaceHeight / 2.0f;
                    float length = sqrtf((float)surfaceWidth * surfaceWidth + surfaceHeight * surfaceHeight);
                    
                    D2D1_POINT_2F start = D2D1::Point2F(cx - cosf(rad) * length / 2, cy - sinf(rad) * length / 2);
                    D2D1_POINT_2F end = D2D1::Point2F(cx + cosf(rad) * length / 2, cy + sinf(rad) * length / 2);

                    ID2D1LinearGradientBrush* pBrush = nullptr;
                    dc->CreateLinearGradientBrush(D2D1::LinearGradientBrushProperties(start, end), pStopCollection, &pBrush);

                    if (pBrush) {
                        D2D1_ROUNDED_RECT roundedRect = D2D1::RoundedRect(
                            D2D1::RectF((float)data.borderSize / 2, (float)data.borderSize / 2, 
                                        (float)surfaceWidth - data.borderSize / 2, (float)surfaceHeight - data.borderSize / 2),
                            data.rounding, data.rounding
                        );
                        
                        dc->DrawRoundedRectangle(roundedRect, (ID2D1Brush*)pBrush, (float)data.borderSize);
                        pBrush->Release();
                    }
                    pStopCollection->Release();
                }
                
                surface->EndDraw();
                dc->Release();
            }
            
            data.visual->SetContent(surface);
            surface->Release();
        }

        g_State.dcompDevice->Commit();
    }
}
