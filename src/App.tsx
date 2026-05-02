import { Button } from "@/components/ui/button";

function App() {
  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="text-center space-y-4">
        <h1 className="text-4xl font-bold text-foreground">MisakaX</h1>
        <p className="text-muted-foreground">Desktop AI Agent Client</p>
        <Button variant="default" size="lg">
          Get Started
        </Button>
      </div>
    </div>
  );
}

export default App;
