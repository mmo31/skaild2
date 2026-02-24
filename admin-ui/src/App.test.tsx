import React from 'react';
import { render, screen, within } from '@testing-library/react';
import App from './App';

describe('App shell', () => {
  it('renders the core navigation sections', () => {
    render(<App />);

    const nav = screen.getByRole('navigation', { name: /main navigation/i });
    const { getByText } = within(nav);

    expect(getByText('Dashboard')).toBeInTheDocument();
    expect(getByText('Routes')).toBeInTheDocument();
    expect(getByText('Identity')).toBeInTheDocument();
    expect(getByText('Certificates')).toBeInTheDocument();
    expect(getByText('Settings')).toBeInTheDocument();
  });

  it('shows primary admin dashboard actions', () => {
    render(<App />);

    expect(screen.getByText('Add Route')).toBeInTheDocument();
    expect(screen.getByText('Connect IdP')).toBeInTheDocument();
  });
});
