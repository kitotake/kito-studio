import { Component, type ErrorInfo, type ReactNode } from "react";

type ErrorBoundaryProps = { children: ReactNode };
type ErrorBoundaryState = { error: Error | null };

/** Dernier filet de sécurité : évite l'écran blanc en cas d'erreur de rendu React. */
export class ErrorBoundary extends Component<ErrorBoundaryProps, ErrorBoundaryState> {
  state: ErrorBoundaryState = { error: null };

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error("[Kito Studio] Erreur d'interface non gérée", error, info.componentStack);
  }

  render() {
    const { error } = this.state;
    if (error === null) return this.props.children;

    return (
      <main className="fatal-error">
        <section className="fatal-error-card" role="alert">
          <div className="eyebrow">ERREUR INATTENDUE</div>
          <h1>Kito Studio a rencontré un problème</h1>
          <p>L'interface ne peut pas continuer. Vos médias originaux ne sont jamais modifiés.</p>
          <pre>{error.message}</pre>
          <button className="primary-button" onClick={() => window.location.reload()}>
            Recharger l'application
          </button>
        </section>
      </main>
    );
  }
}
