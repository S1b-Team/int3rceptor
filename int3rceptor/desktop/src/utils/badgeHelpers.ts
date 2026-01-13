// Helper functions for Badge colors

type BadgeVariant = 'cyan' | 'magenta' | 'orange' | 'purple' | 'gray'

// Helper function for HTTP method colors
export function getMethodVariant(method: string): BadgeVariant {
  const methodMap: Record<string, BadgeVariant> = {
    'GET': 'cyan',
    'POST': 'magenta',
    'PUT': 'orange',
    'DELETE': 'magenta',
    'PATCH': 'purple',
    'OPTIONS': 'gray',
    'HEAD': 'cyan',
  }

  return methodMap[method.toUpperCase()] || 'gray'
}

// Helper function for status code colors
export function getStatusVariant(status: number): BadgeVariant {
  if (status >= 200 && status < 300) return 'cyan'
  if (status >= 300 && status < 400) return 'purple'
  if (status >= 400 && status < 500) return 'magenta'
  if (status >= 500) return 'orange'
  return 'gray'
}
