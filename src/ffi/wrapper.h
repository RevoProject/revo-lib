#if __has_include(<obs/obs.h>)
#include <obs/obs.h>
#if __has_include(<obs/obs-frontend-api.h>)
#include <obs/obs-frontend-api.h>
#endif
#if __has_include(<obs/util/base.h>)
#include <obs/util/base.h>
#endif
#if __has_include(<obs/graphics/graphics.h>)
#include <obs/graphics/graphics.h>
#endif
#else
#include <obs.h>
#if __has_include(<obs-frontend-api.h>)
#include <obs-frontend-api.h>
#endif
#if __has_include(<util/base.h>)
#include <util/base.h>
#endif
#if __has_include(<graphics/graphics.h>)
#include <graphics/graphics.h>
#endif
#endif

#include <stdio.h>
